use tower_http::cors::CorsLayer;

pub fn cors() -> CorsLayer {
    CorsLayer::permissive()
}
