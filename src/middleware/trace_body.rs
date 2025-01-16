use super::{DEFAULT_ERROR_LEVEL, DEFAULT_MESSAGE_LEVEL};
use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
    response::Response,
};
use futures_util::future::BoxFuture;
use http_body_util::BodyExt;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::Level;

macro_rules! event_dynamic_lvl {
    ($level:expr, $($arg:tt)+) => {
        match $level {
            tracing::Level::ERROR => {
                tracing::event!(tracing::Level::ERROR, $($arg)+);
            }
            tracing::Level::WARN => {
                tracing::event!(tracing::Level::WARN, $($arg)+);
            }
            tracing::Level::INFO => {
                tracing::event!(tracing::Level::INFO, $($arg)+);
            }
            tracing::Level::DEBUG => {
                tracing::event!(tracing::Level::DEBUG, $($arg)+);
            }
            tracing::Level::TRACE => {
                tracing::event!(tracing::Level::TRACE, $($arg)+);
            }
        }
    };
}

#[derive(Debug, Clone)]
pub struct TraceBodyLayer {
    level: Level,
}

impl TraceBodyLayer {
    pub fn new() -> Self {
        Self {
            level: DEFAULT_MESSAGE_LEVEL,
        }
    }

    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }
}

impl Default for TraceBodyLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for TraceBodyLayer {
    type Service = TraceBody<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TraceBody {
            inner,
            level: self.level,
        }
    }
}

#[derive(Clone)]
pub struct TraceBody<S> {
    inner: S,
    level: Level,
}

impl<S> Service<Request<Body>> for TraceBody<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        let level = self.level;
        Box::pin(async move {
            let (parts, body) = request.into_parts();
            let bytes = match collect_and_log("request", body, level).await {
                Ok(bytes) => bytes,
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from("Bad Request"))
                        .unwrap());
                }
            };
            let request = Request::from_parts(parts, Body::from(bytes));

            let response = inner.call(request).await?;

            let (parts, body) = response.into_parts();
            let bytes = match collect_and_log("response", body, level).await {
                Ok(bytes) => bytes,
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Internal Server Error"))
                        .unwrap());
                }
            };
            let response = Response::from_parts(parts, Body::from(bytes));

            Ok(response)
        })
    }
}

async fn collect_and_log<B>(direction: &str, body: B, level: Level) -> Result<Bytes, B::Error>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            event_dynamic_lvl!(
                DEFAULT_ERROR_LEVEL,
                "failed to read {direction} body: {err}"
            );
            return Err(err);
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        event_dynamic_lvl!(level, "{direction} body = {body:?}");
    }

    Ok(bytes)
}

pub fn trace_body() -> TraceBodyLayer {
    TraceBodyLayer::default()
}
