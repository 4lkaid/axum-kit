use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
    response::Response,
};
use futures_util::future::BoxFuture;
use http_body_util::BodyExt;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct TraceBodyLayer;

impl<S> Layer<S> for TraceBodyLayer {
    type Service = TraceBody<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TraceBody { inner }
    }
}

#[derive(Clone)]
pub struct TraceBody<S> {
    inner: S,
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
        Box::pin(async move {
            let (parts, body) = request.into_parts();
            let bytes = match collect_and_log("request", body).await {
                Ok(bytes) => bytes,
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from("Bad Request"))
                        .unwrap());
                }
            };
            let request = Request::from_parts(parts, Body::from(bytes));

            let response = match inner.call(request).await {
                Ok(response) => response,
                Err(err) => return Err(err),
            };

            let (parts, body) = response.into_parts();
            let bytes = match collect_and_log("response", body).await {
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

async fn collect_and_log<B>(direction: &str, body: B) -> Result<Bytes, B::Error>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            tracing::error!("failed to read {direction} body: {err}");
            return Err(err);
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{direction} body = {body:?}");
    }

    Ok(bytes)
}

pub fn trace_body() -> TraceBodyLayer {
    TraceBodyLayer
}
