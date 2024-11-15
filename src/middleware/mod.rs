pub mod cors;
pub mod request_id;
pub mod request_response_logger;
pub mod trace;

pub const DIRECT_CONNECT_IP: &str = "direct-connect-ip";
pub const X_FORWARDED_FOR: &str = "x-forwarded-for";
pub const X_REAL_IP: &str = "x-real-ip";
pub const X_REQUEST_ID: &str = "x-request-id";
