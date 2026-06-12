use appmo_core::{
    AppError, AppInstallRequest, AppLaunchRequest, AppTerminateRequest, DeviceId, DeviceManager,
    KeyRequest, LogRequest, ProcessRunner, RecordRequest, SwipeRequest, TapRequest, TextRequest,
};
use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use futures_util::stream;
use image::{codecs::jpeg::JpegEncoder, imageops::FilterType, GenericImageView};
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    net::SocketAddr,
    time::{Duration, Instant},
};
use tokio::net::UdpSocket;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct AppState<R: ProcessRunner + Clone> {
    controller: DeviceManager<R>,
}

pub fn build_router<R: ProcessRunner + Clone>(controller: DeviceManager<R>) -> Router {
    let state = AppState { controller };
    let api = Router::new()
        .route("/devices", get(list_devices::<R>))
        .route("/devices/:id/screenshot", get(screenshot::<R>))
        .route(
            "/devices/:id/screenshot-stream",
            get(screenshot_stream::<R>),
        )
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

#[derive(Debug, Deserialize)]
struct StreamRequest {
    fps: Option<u32>,
    format: Option<String>,
    max_width: Option<u32>,
    quality: Option<u8>,
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
    let screenshot = state.controller.screenshot(&id).await?;
    Ok((
        [(header::CONTENT_TYPE, screenshot.content_type)],
        screenshot.bytes,
    )
        .into_response())
}

async fn screenshot_stream<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    Query(req): Query<StreamRequest>,
) -> Result<Response, ApiError> {
    let id = DeviceId::parse(&id)?;
    let fps = req.fps.unwrap_or(8).clamp(1, 15);
    let encode_jpeg = req
        .format
        .as_deref()
        .is_some_and(|format| format.eq_ignore_ascii_case("jpeg"));
    let max_width = req.max_width.unwrap_or(720).clamp(240, 4096);
    let quality = req.quality.unwrap_or(70).clamp(35, 95);
    let frame_delay = Duration::from_millis(1_000 / u64::from(fps));
    let controller = state.controller.clone();
    let boundary = "appmo-frame";
    let body_stream = stream::unfold(
        (controller, id, None::<Instant>),
        move |(controller, id, last_frame)| async move {
            if let Some(last_frame) = last_frame {
                let elapsed = last_frame.elapsed();
                if elapsed < frame_delay {
                    tokio::time::sleep(frame_delay - elapsed).await;
                }
            }
            let frame_started = Instant::now();
            match controller.screenshot(&id).await {
                Ok(screenshot) => {
                    let frame = if encode_jpeg {
                        encode_stream_frame(&screenshot.bytes, max_width, quality)
                            .unwrap_or_else(|_| (screenshot.content_type, screenshot.bytes))
                    } else {
                        (screenshot.content_type, screenshot.bytes)
                    };
                    let header = format!(
                    "\r\n--{boundary}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n\r\n",
                    frame.0,
                    frame.1.len()
                );
                    let mut payload = Vec::with_capacity(header.len() + frame.1.len());
                    payload.extend_from_slice(header.as_bytes());
                    payload.extend_from_slice(&frame.1);
                    Some((
                        Ok::<Bytes, Infallible>(Bytes::from(payload)),
                        (controller, id, Some(frame_started)),
                    ))
                }
                Err(error) => {
                    warn!(device = %id.web_id(), %error, "screenshot stream stopped");
                    None
                }
            }
        },
    );

    Ok((
        [
            (
                header::CONTENT_TYPE,
                format!("multipart/x-mixed-replace; boundary={boundary}"),
            ),
            (header::CACHE_CONTROL, "no-store".to_string()),
        ],
        Body::from_stream(body_stream),
    )
        .into_response())
}

fn encode_stream_frame(
    bytes: &[u8],
    max_width: u32,
    quality: u8,
) -> Result<(&'static str, Vec<u8>), image::ImageError> {
    let image = image::load_from_memory(bytes)?;
    let (width, height) = image.dimensions();
    let output = if width > max_width {
        let scaled_height = ((u64::from(height) * u64::from(max_width)) / u64::from(width))
            .max(1)
            .min(u64::from(u32::MAX)) as u32;
        image.resize(max_width, scaled_height, FilterType::Triangle)
    } else {
        image
    };

    let mut encoded = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut encoded, quality);
    encoder.encode_image(&output)?;
    Ok(("image/jpeg", encoded))
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
    State(state): State<AppState<R>>,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state)))
}

async fn handle_socket<R: ProcessRunner + Clone>(mut socket: WebSocket, state: AppState<R>) {
    let _ = socket
        .send(Message::Text("Appmo WebSocket connected".to_string()))
        .await;
    while let Some(Ok(message)) = socket.recv().await {
        match message {
            Message::Text(text) => {
                let response = handle_ws_text(&state, &text).await;
                if socket.send(Message::Text(response)).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

async fn handle_ws_text<R: ProcessRunner + Clone>(state: &AppState<R>, text: &str) -> String {
    let request = match serde_json::from_str::<ControlRequest>(text) {
        Ok(request) => request,
        Err(error) => {
            return ControlResponse::error(None, format!("invalid control message: {error}"))
                .to_json()
        }
    };
    let request_id = Some(request.request_id.clone());
    let result = execute_control(state, request).await;
    match result {
        Ok(()) => ControlResponse::ok(request_id).to_json(),
        Err(error) => ControlResponse::error(request_id, error.0.to_string()).to_json(),
    }
}

async fn execute_control<R: ProcessRunner + Clone>(
    state: &AppState<R>,
    request: ControlRequest,
) -> Result<(), ApiError> {
    let id = DeviceId::parse(&request.device_id)?;
    match request.command {
        ControlCommand::Tap {
            x,
            y,
            source_width,
            source_height,
        } => {
            state
                .controller
                .tap(
                    &id,
                    TapRequest {
                        x,
                        y,
                        source_width,
                        source_height,
                    },
                )
                .await?
        }
        ControlCommand::Swipe {
            x1,
            y1,
            x2,
            y2,
            duration_ms,
            source_width,
            source_height,
        } => {
            state
                .controller
                .swipe(
                    &id,
                    SwipeRequest {
                        x1,
                        y1,
                        x2,
                        y2,
                        duration_ms,
                        source_width,
                        source_height,
                    },
                )
                .await?
        }
        ControlCommand::Key { key } => state.controller.key(&id, KeyRequest { key }).await?,
        ControlCommand::Text { text } => state.controller.text(&id, TextRequest { text }).await?,
    }
    Ok(())
}

pub async fn run_udp_control<R: ProcessRunner + Clone>(
    controller: DeviceManager<R>,
    bind: SocketAddr,
) -> std::io::Result<()> {
    let socket = UdpSocket::bind(bind).await?;
    info!(%bind, "appmo udp control listening");
    let state = AppState { controller };
    let mut buffer = vec![0_u8; 65_507];

    loop {
        let (len, peer) = socket.recv_from(&mut buffer).await?;
        let text = match std::str::from_utf8(&buffer[..len]) {
            Ok(text) => text,
            Err(error) => {
                let response =
                    ControlResponse::error(None, format!("invalid utf-8 datagram: {error}"))
                        .to_json();
                let _ = socket.send_to(response.as_bytes(), peer).await;
                continue;
            }
        };
        let response = handle_udp_text(&state, text, peer).await;
        if let Err(error) = socket.send_to(response.as_bytes(), peer).await {
            warn!(%peer, %error, "failed to send udp control response");
        }
    }
}

async fn handle_udp_text<R: ProcessRunner + Clone>(
    state: &AppState<R>,
    text: &str,
    peer: SocketAddr,
) -> String {
    let request = match serde_json::from_str::<ControlRequest>(text) {
        Ok(request) => request,
        Err(error) => {
            return ControlResponse::error(None, format!("invalid control datagram: {error}"))
                .to_json()
        }
    };
    let request_id = Some(request.request_id.clone());
    let result = execute_control(state, request).await;
    match result {
        Ok(()) => {
            info!(%peer, "udp control command completed");
            ControlResponse::ok(request_id).to_json()
        }
        Err(error) => {
            error!(%peer, error = %error.0, "udp control command failed");
            ControlResponse::error(request_id, error.0.to_string()).to_json()
        }
    }
}

#[derive(Debug, Deserialize)]
struct ControlRequest {
    request_id: String,
    device_id: String,
    #[serde(flatten)]
    command: ControlCommand,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ControlCommand {
    Tap {
        x: u32,
        y: u32,
        source_width: Option<u32>,
        source_height: Option<u32>,
    },
    Swipe {
        x1: u32,
        y1: u32,
        x2: u32,
        y2: u32,
        duration_ms: Option<u32>,
        source_width: Option<u32>,
        source_height: Option<u32>,
    },
    Key {
        key: String,
    },
    Text {
        text: String,
    },
}

#[derive(Debug, Serialize)]
struct ControlResponse {
    #[serde(rename = "type")]
    kind: &'static str,
    request_id: Option<String>,
    ok: bool,
    error: Option<String>,
}

impl ControlResponse {
    fn ok(request_id: Option<String>) -> Self {
        Self {
            kind: "control_result",
            request_id,
            ok: true,
            error: None,
        }
    }

    fn error(request_id: Option<String>, error: String) -> Self {
        Self {
            kind: "control_result",
            request_id,
            ok: false,
            error: Some(error),
        }
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"type":"control_result","request_id":null,"ok":false,"error":"serialization failed"}"#
                .to_string()
        })
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
            udp_bind: None,
            adb_path: "/bin/echo".into(),
            xcrun_path: "/bin/echo".into(),
            osascript_path: "/bin/echo".into(),
            idb_path: "/bin/echo".into(),
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
