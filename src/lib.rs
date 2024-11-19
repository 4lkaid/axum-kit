pub mod config;
pub mod error;
pub mod general;
pub mod logger;
pub mod middleware;
pub mod validation;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "redis")]
pub mod redis;

pub type AppResult<T> = Result<T, error::Error>;
