use appmo_core::{
    AppError, AppInstallRequest, AppLaunchRequest, AppTerminateRequest, DeviceId, DeviceManager,
    KeyRequest, LogRequest, ProcessRunner, RecordRequest, SwipeRequest, TapRequest, TextRequest,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState<R: ProcessRunner + Clone> {
    controller: DeviceManager<R>,
}

pub fn build_router<R: ProcessRunner + Clone>(controller: DeviceManager<R>) -> Router {
    let state = AppState { controller };
    let api = Router::new()
        .route("/devices", get(list_devices::<R>))
        .route("/devices/:id/screenshot", get(screenshot::<R>))
        .route("/devices/:id/input/tap", post(tap::<R>))
        .route("/devices/:id/input/swipe", post(swipe::<R>))
        .route("/devices/:id/input/text", post(text::<R>))
        .route("/devices/:id/key", post(key::<R>))
        .route("/devices/:id/app/install", post(install::<R>))
        .route("/devices/:id/app/launch", post(launch::<R>))
        .route("/devices/:id/app/terminate", post(terminate::<R>))
        .route("/devices/:id/logs", get(logs::<R>))
        .route("/devices/:id/record/start", post(record_start::<R>))
        .route("/devices/:id/record/stop", post(record_stop::<R>));

    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/ws", get(ws_handler::<R>))
        .nest("/api", api)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

async fn index() -> Html<String> {
    Html(appmo_web::dashboard_html().to_string())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

async fn list_devices<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let devices = state.controller.list_devices().await?;
    Ok(Json(serde_json::to_value(devices)?))
}

async fn screenshot<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let id = DeviceId::parse(&id)?;
    let bytes = state.controller.screenshot(&id).await?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes).into_response())
}

async fn tap<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    Json(req): Json<TapRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let id = DeviceId::parse(&id)?;
    state.controller.tap(&id, req).await?;
    Ok(Json(CommandResponse::ok()))
}

async fn swipe<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    Json(req): Json<SwipeRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let id = DeviceId::parse(&id)?;
    state.controller.swipe(&id, req).await?;
    Ok(Json(CommandResponse::ok()))
}

async fn text<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    Json(req): Json<TextRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let id = DeviceId::parse(&id)?;
    state.controller.text(&id, req).await?;
    Ok(Json(CommandResponse::ok()))
}

async fn key<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    Json(req): Json<KeyRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let id = DeviceId::parse(&id)?;
    state.controller.key(&id, req).await?;
    Ok(Json(CommandResponse::ok()))
}

async fn install<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    Json(req): Json<AppInstallRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let id = DeviceId::parse(&id)?;
    state.controller.install(&id, req).await?;
    Ok(Json(CommandResponse::ok()))
}

async fn launch<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    Json(req): Json<AppLaunchRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let id = DeviceId::parse(&id)?;
    state.controller.launch(&id, req).await?;
    Ok(Json(CommandResponse::ok()))
}

async fn terminate<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    Json(req): Json<AppTerminateRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let id = DeviceId::parse(&id)?;
    state.controller.terminate(&id, req).await?;
    Ok(Json(CommandResponse::ok()))
}

async fn logs<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    Query(req): Query<LogRequest>,
) -> Result<Response, ApiError> {
    let id = DeviceId::parse(&id)?;
    let logs = state.controller.logs(&id, req).await?;
    Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], logs).into_response())
}

async fn record_start<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    Json(req): Json<RecordRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = DeviceId::parse(&id)?;
    let path = state.controller.record_start(&id, req).await?;
    Ok(Json(serde_json::json!({ "ok": true, "path": path })))
}

async fn record_stop<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = DeviceId::parse(&id)?;
    let path = state.controller.record_stop(&id).await?;
    Ok(Json(serde_json::json!({ "ok": true, "path": path })))
}

async fn ws_handler<R: ProcessRunner + Clone>(
    State(_state): State<AppState<R>>,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    Ok(ws.on_upgrade(handle_socket))
}

async fn handle_socket(mut socket: WebSocket) {
    let _ = socket
        .send(Message::Text("Appmo WebSocket connected".to_string()))
        .await;
    while let Some(Ok(message)) = socket.recv().await {
        if matches!(message, Message::Close(_)) {
            break;
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    ok: bool,
}

impl CommandResponse {
    fn ok() -> Self {
        Self { ok: true }
    }
}

pub struct ApiError(AppError);

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(value: serde_json::Error) -> Self {
        Self(AppError::Json(value))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            AppError::DeviceNotFound(_) => StatusCode::NOT_FOUND,
            AppError::ToolMissing(_) => StatusCode::FAILED_DEPENDENCY,
            AppError::UnsupportedCapability(_) => StatusCode::NOT_IMPLEMENTED,
            AppError::CommandFailed { .. } => StatusCode::BAD_GATEWAY,
            AppError::Io(_) | AppError::Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = serde_json::json!({
            "error": self.0.to_string()
        });
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use std::{path::Path, sync::Arc};

    use appmo_core::{AppConfig, AppResult, CommandOutput};
    use async_trait::async_trait;
    use axum::{body::Body, http::Request};
    use tokio::sync::Mutex;
    use tower::ServiceExt;

    use super::*;

    #[derive(Clone, Default)]
    struct MockRunner {
        calls: Arc<Mutex<Vec<Vec<String>>>>,
    }

    #[async_trait]
    impl ProcessRunner for MockRunner {
        async fn run(&self, _program: &Path, args: &[String]) -> AppResult<CommandOutput> {
            self.calls.lock().await.push(args.to_vec());
            let stdout = if args == ["devices".to_string(), "-l".to_string()] {
                b"List of devices attached\nemulator-5554 device model:Pixel_8\n".to_vec()
            } else if args
                == [
                    "simctl".to_string(),
                    "list".to_string(),
                    "devices".to_string(),
                    "--json".to_string(),
                ]
            {
                br#"{"devices":{"iOS":[{"name":"iPhone","udid":"ABC","state":"Booted","isAvailable":true}]}}"#.to_vec()
            } else {
                Vec::new()
            };
            Ok(CommandOutput {
                status: Some(0),
                stdout,
                stderr: String::new(),
            })
        }
    }

    fn test_router() -> Router {
        let config = AppConfig {
            bind: "127.0.0.1:0".parse().unwrap(),
            adb_path: "/bin/echo".into(),
            xcrun_path: "/bin/echo".into(),
        };
        build_router(DeviceManager::with_runner(config, MockRunner::default()))
    }

    #[tokio::test]
    async fn health_is_public() {
        let response = test_router()
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn api_is_public() {
        let response = test_router()
            .oneshot(Request::get("/api/devices").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
