use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::StatusCode,
    middleware::{FromFnLayer, Next},
    response::Response,
};
use http_body_util::BodyExt;
use std::future::Future;

pub async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{direction} body = {body:?}");
    }

    Ok(bytes)
}

pub trait MyFn<R, N>: Fn(R, N) -> <Self as MyFn<R, N>>::Output {
    type Output;
}

impl<F, R, N, Out> MyFn<R, N> for F
where
    F: Fn(R, N) -> Out,
{
    type Output = Out;
}

#[allow(clippy::type_complexity)]
pub fn print() -> FromFnLayer<
    impl MyFn<
            Request,
            Next,
            Output: Future<Output = Result<Response, (StatusCode, String)>> + Send + 'static,
        > + Copy
        + Send
        + Sync
        + 'static,
    (),
    (Request,),
> {
    axum::middleware::from_fn(print_request_response)
}
