use appmon_core::{AppConfig, AppController};
use appmon_server::{build_router, run_udp_control};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod bootstrap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "appmon_server=info,appmon_core=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    bootstrap::ensure_runtime_dependencies();

    let config = AppConfig::from_env()?;
    let bind = config.bind;
    let udp_bind = config.udp_bind;
    let controller = AppController::new(config);
    if let Some(udp_bind) = udp_bind {
        let udp_controller = controller.clone();
        tokio::spawn(async move {
            if let Err(error) = run_udp_control(udp_controller, udp_bind).await {
                tracing::error!(%udp_bind, %error, "appmon udp control stopped");
            }
        });
    }
    let app = build_router(controller);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(%bind, "appmon server listening");
    axum::serve(listener, app).await?;
    Ok(())
}
