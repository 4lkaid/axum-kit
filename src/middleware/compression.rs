use tower_http::compression::CompressionLayer;

pub fn compression() -> CompressionLayer {
    CompressionLayer::new()
}
