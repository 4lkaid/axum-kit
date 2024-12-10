#[cfg(feature = "postgres")]
use crate::postgres;

#[cfg(feature = "redis")]
use crate::redis;

use crate::{
    config::{load_config, Config},
    general, logger,
};
use anyhow::{Context, Result};
use axum::Router;
use tracing_appender::non_blocking::WorkerGuard;

type TaskHandle = tokio::task::JoinHandle<Result<()>>;

pub struct Application {
    config: Config,
    router: Router,
    pre_run_fn: Option<Box<dyn FnOnce() -> TaskHandle + Send + Sync>>,
}

impl Application {
    pub fn default(config_path: &str, router: Router) -> Result<Self> {
        let config = load_config(config_path).with_context(|| "configuration parsing failed")?;
        Ok(Self {
            config,
            router,
            pre_run_fn: None,
        })
    }

    pub fn new(config: Config, router: Router) -> Result<Self> {
        Ok(Self {
            config,
            router,
            pre_run_fn: None,
        })
    }

    pub fn before_run<F>(mut self, callback: F) -> Self
    where
        F: FnOnce() -> TaskHandle + Send + Sync + 'static,
    {
        self.pre_run_fn = Some(Box::new(callback));
        self
    }

    pub async fn run(self) -> Result<WorkerGuard> {
        #[cfg(feature = "postgres")]
        postgres::init(&self.config.postgres)
            .await
            .with_context(|| "postgres initialization failed")?;

        #[cfg(feature = "redis")]
        redis::init(&self.config.redis)
            .await
            .with_context(|| "redis initialization failed")?;

        if let Some(callback) = self.pre_run_fn {
            let _ = callback().await?;
        }
        let worker_guard =
            logger::init(&self.config.logger).with_context(|| "logger initialization failed")?;
        general::serve(&self.config.general, self.router)
            .await
            .with_context(|| "service startup failed")?;

        Ok(worker_guard)
    }
}
