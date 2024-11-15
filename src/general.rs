use anyhow::Result;
use axum::Router;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub listen: String,
}

pub async fn serve(config: &GeneralConfig, router: Router) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(&config.listen).await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
