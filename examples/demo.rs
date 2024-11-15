//! Run with
//!
//! ```not_rust
//! sqlx database create
//! sqlx migrate run --source ./examples
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
    use ::redis::AsyncCommands;
    use axum::extract::Json;
    use axum_kit::{postgres, redis, validation::ValidatedJson, AppResult};
    use serde::{Deserialize, Serialize};
    use validator::Validate;

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
        let mut con = redis::conn().await?;
        let _: () = con.set_ex("greeting", "Hello, AXUM-KIT!", 10).await?;
        let result: String = con.get("greeting").await?;
        Ok(result)
    }

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
    axum_kit::postgres::init(&config.postgres).await?;
    axum_kit::redis::init(&config.redis).await?;
    let _worker_guard =
        axum_kit::logger::init(&config.logger).with_context(|| "initialization failed")?;
    let router = route::init();
    axum_kit::general::serve(&config.general, router)
        .await
        .with_context(|| "service startup failed")?;
    Ok(())
}
