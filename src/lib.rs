pub mod config;
pub mod error;
pub mod general;
pub mod logger;
pub mod middleware;
pub mod postgres;
pub mod redis;
pub mod validation;

pub type AppResult<T> = Result<T, error::Error>;
