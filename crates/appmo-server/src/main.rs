use appmo_core::{AppConfig, AppController};
use appmo_server::build_router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "appmo_server=info,appmo_core=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env()?;
    let bind = config.bind;
    let controller = AppController::new(config);
    let app = build_router(controller);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(%bind, "appmo server listening");
    axum::serve(listener, app).await?;
    Ok(())
}
