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

    pub async fn root() -> AppResult<String> {
        #[cfg(feature = "redis")]
        {
            let mut con = redis::conn().await?;
            let _: () = con
                .set_ex("greeting", "Hello, Axum-kit with Redis!", 10)
                .await?;
            let result: String = con.get("greeting").await?;
            Ok(result)
        }
        #[cfg(not(feature = "redis"))]
        Ok("Hello, Axum-kit without Redis!".to_string())
    }

    pub async fn create_user(
        ValidatedJson(payload): ValidatedJson<CreateUser>,
    ) -> AppResult<Json<User>> {
        #[cfg(feature = "postgres")]
        {
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
        {
            let user = User {
                id: 9527,
                username: payload.username,
            };
            Ok(Json(user))
        }
    }
}

mod route {
    use crate::handler;
    use axum::{
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
                    .layer(request_response_logger::print()),
            )
    }
}

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let router = route::init();
    let _worker_guard = axum_kit::bootstrap::Application::default("config.toml", router)?
        .before_run(|| {
            tokio::spawn(async move {
                println!("Running pre-run initialization tasks...");
                Ok(())
            })
        })
        .run()
        .await?;
    Ok(())
}
