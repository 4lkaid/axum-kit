use anyhow::Result;
use serde::Deserialize;
use std::io::Write;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{time::ChronoLocal, writer::MakeWriterExt},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[derive(Debug, Deserialize)]
pub struct LoggerConfig {
    pub level: LogLevel,
    pub writer: LogWriter,
    pub directory: String,
    pub file_name_prefix: String,
}

#[derive(Debug, Deserialize)]
pub enum LogLevel {
    #[serde(rename = "trace")]
    Trace,
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Deserialize)]
pub enum LogWriter {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "stdout")]
    Stdout,
}

impl LogLevel {
    pub fn to_tracing_level(&self) -> Level {
        match self {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

pub fn init(config: &LoggerConfig) -> Result<WorkerGuard> {
    let (writer, ansi): (Box<dyn Write + Send + 'static>, bool) = match config.writer {
        LogWriter::File => (
            Box::new(tracing_appender::rolling::daily(
                config.directory.as_str(),
                config.file_name_prefix.as_str(),
            )),
            false,
        ),

        LogWriter::Stdout => (Box::new(std::io::stdout()), true),
    };
    let (non_blocking, worker_guard) = tracing_appender::non_blocking(writer);
    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(ansi)
        .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string()))
        .with_writer(non_blocking.with_max_level(config.level.to_tracing_level()));
    tracing_subscriber::registry().with(layer).init();
    Ok(worker_guard)
}
