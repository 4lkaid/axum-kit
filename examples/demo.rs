//! Run with
//!
//! ```not_rust
//! sqlx database create
//! sqlx migrate run --source ./examples
//! # With optional features
//! cargo run --example demo --features postgres,redis
//! # Or without features
//! cargo run --example demo
//! ```
//!
//! Test with curl:
//!
//! ```not_rust
//! curl 127.0.0.1:8000
//! curl -X POST -H 'Content-Type:application/json' -d '{"username":"axum-kit"}' 127.0.0.1:8000/users
//! ```

mod handler {
    use axum::extract::Json;
    use axum_kit::{validation::ValidatedJson, AppResult};
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    #[cfg(feature = "postgres")]
    use axum_kit::postgres;

    #[cfg(feature = "redis")]
    use ::redis::AsyncCommands;

    #[cfg(feature = "redis")]
    use axum_kit::redis;

    #[derive(Deserialize, Validate)]
    pub struct CreateUser {
        #[validate(length(min = 1, message = "Can not be empty"))]
        pub username: String,
    }

    #[derive(Serialize)]
    pub struct User {
        pub id: i64,
        pub username: String,
    }

    #[cfg(feature = "redis")]
    pub async fn root() -> AppResult<String> {
        let mut con = redis::conn().await?;
        let _: () = con
            .set_ex("greeting", "Hello, Axum-kit with Redis!", 10)
            .await?;
        let result: String = con.get("greeting").await?;
        Ok(result)
    }

    #[cfg(not(feature = "redis"))]
    pub async fn root() -> AppResult<String> {
        Ok("Hello, Axum-kit without Redis!".to_string())
    }

    #[cfg(feature = "postgres")]
    pub async fn create_user(
        ValidatedJson(payload): ValidatedJson<CreateUser>,
    ) -> AppResult<Json<User>> {
        let user = sqlx::query_as!(
            User,
            r#"insert into users (username) values ($1) returning id, username"#,
            payload.username
        )
        .fetch_one(postgres::conn())
        .await?;
        Ok(Json(user))
    }

    #[cfg(not(feature = "postgres"))]
    pub async fn create_user(
        ValidatedJson(payload): ValidatedJson<CreateUser>,
    ) -> AppResult<Json<User>> {
        let user = User {
            id: 9527,
            username: payload.username,
        };
        Ok(Json(user))
    }
}

mod route {
    use crate::handler;
    use axum::{
        middleware,
        routing::{get, post},
        Router,
    };
    use axum_kit::middleware::{cors, request_id, request_response_logger, trace};
    use tower::ServiceBuilder;

    pub fn init() -> Router {
        Router::new()
            .route("/", get(handler::root))
            .route("/users", post(handler::create_user))
            .layer(
                ServiceBuilder::new()
                    .layer(request_id::set_request_id())
                    .layer(request_id::propagate_request_id())
                    .layer(trace::trace())
                    .layer(cors::cors())
                    .layer(middleware::from_fn(
                        request_response_logger::print_request_response,
                    )),
            )
    }
}

use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let config = axum_kit::config::load_config().with_context(|| "configuration parsing failed")?;

    #[cfg(feature = "postgres")]
    axum_kit::postgres::init(&config.postgres)
        .await
        .with_context(|| "postgres initialization failed")?;

    #[cfg(feature = "redis")]
    axum_kit::redis::init(&config.redis)
        .await
        .with_context(|| "redis initialization failed")?;

    let _worker_guard =
        axum_kit::logger::init(&config.logger).with_context(|| "logger initialization failed")?;
    let router = route::init();
    axum_kit::general::serve(&config.general, router)
        .await
        .with_context(|| "service startup failed")?;
    Ok(())
}
