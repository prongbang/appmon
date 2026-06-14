use appmon_core::{
    android_grpc, AppError, AppInstallRequest, AppLaunchRequest, AppTerminateRequest, DeviceId,
    DeviceKind, DeviceManager, KeyRequest, LogRequest, MotionAction, MotionRequest, ProcessRunner,
    RecordRequest, SwipeRequest, TapRequest, TextRequest,
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
    collections::HashMap,
    convert::Infallible,
    net::SocketAddr,
    process::Stdio,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UdpSocket,
    process::Command,
    sync::Mutex,
};
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use uuid::Uuid;
use webrtc::media::Sample;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::{MediaEngine, MIME_TYPE_VP8},
        APIBuilder,
    },
    data_channel::{data_channel_state::RTCDataChannelState, RTCDataChannel},
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription, RTCPeerConnection,
    },
    rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
    track::track_local::track_local_static_sample::TrackLocalStaticSample,
};

#[derive(Clone)]
pub struct AppState<R: ProcessRunner + Clone> {
    controller: DeviceManager<R>,
    webrtc_sessions: Arc<Mutex<HashMap<Uuid, Arc<RTCPeerConnection>>>>,
}

pub fn build_router<R: ProcessRunner + Clone>(controller: DeviceManager<R>) -> Router {
    let state = AppState {
        controller,
        webrtc_sessions: Arc::new(Mutex::new(HashMap::new())),
    };
    let api = Router::new()
        .route("/devices", get(list_devices::<R>))
        .route("/devices/:id/screenshot", get(screenshot::<R>))
        .route(
            "/devices/:id/screenshot-stream",
            get(screenshot_stream::<R>),
        )
        .route("/devices/:id/webrtc/offer", post(webrtc_offer::<R>))
        .route(
            "/devices/:id/emulator-webrtc/ws",
            get(emulator_webrtc_ws::<R>),
        )
        .route("/devices/:id/input/tap", post(tap::<R>))
        .route("/devices/:id/input/swipe", post(swipe::<R>))
        .route("/devices/:id/input/motion", post(motion::<R>))
        .route("/devices/:id/input/text", post(text::<R>))
        .route("/devices/:id/key", post(key::<R>))
        .route("/devices/:id/start", post(start_device::<R>))
        .route("/devices/:id/stop", post(stop_device::<R>))
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
    Html(appmon_web::dashboard_html().to_string())
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

async fn start_device<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = DeviceId::parse(&id)?;
    state.controller.start_device(&id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn stop_device<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = DeviceId::parse(&id)?;
    state.controller.stop_device(&id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn screenshot_stream<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    Query(req): Query<StreamRequest>,
) -> Result<Response, ApiError> {
    let id = DeviceId::parse(&id)?;
    let fps = req.fps.unwrap_or(8).clamp(1, 15);
    let stream_format = req.format.as_deref().unwrap_or("auto").to_ascii_lowercase();
    let max_width = req.max_width.unwrap_or(720).clamp(240, 4096);
    let quality = req.quality.unwrap_or(70).clamp(35, 95);
    let frame_delay = Duration::from_millis(1_000 / u64::from(fps));
    let controller = state.controller.clone();
    let boundary = "appmon-frame";
    let body_stream = stream::unfold(
        (controller, id, None::<Instant>),
        move |(controller, id, last_frame)| {
            let stream_format = stream_format.clone();
            async move {
                if let Some(last_frame) = last_frame {
                    let elapsed = last_frame.elapsed();
                    if elapsed < frame_delay {
                        tokio::time::sleep(frame_delay - elapsed).await;
                    }
                }
                let frame_started = Instant::now();
                match controller.screenshot(&id).await {
                    Ok(screenshot) => {
                        let encode_jpeg = match stream_format.as_str() {
                            "jpeg" => true,
                            "native" => false,
                            _ => {
                                id.kind != DeviceKind::Android
                                    && screenshot.content_type != "image/jpeg"
                            }
                        };
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

#[derive(Debug, Deserialize)]
struct WebRtcOfferRequest {
    offer: RTCSessionDescription,
    transport: Option<WebRtcTransport>,
    fps: Option<u32>,
    format: Option<String>,
    max_width: Option<u32>,
    quality: Option<u8>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum WebRtcTransport {
    Data,
    Media,
}

#[derive(Debug, Serialize)]
struct WebRtcOfferResponse {
    session_id: Uuid,
    answer: RTCSessionDescription,
}

async fn webrtc_offer<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    Json(req): Json<WebRtcOfferRequest>,
) -> Result<Json<WebRtcOfferResponse>, ApiError> {
    let id = DeviceId::parse(&id)?;
    let fps = req.fps.unwrap_or(12).clamp(1, 20);
    let stream_format = req
        .format
        .unwrap_or_else(|| "auto".to_string())
        .to_ascii_lowercase();
    let max_width = req.max_width.unwrap_or(720).clamp(240, 4096);
    let quality = req.quality.unwrap_or(70).clamp(35, 95);
    let session_id = Uuid::new_v4();
    let transport = req.transport.unwrap_or(WebRtcTransport::Data);

    let mut media_engine = MediaEngine::default();
    media_engine
        .register_default_codecs()
        .map_err(webrtc_api_error)?;
    let registry = register_default_interceptors(Registry::new(), &mut media_engine)
        .map_err(webrtc_api_error)?;
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();
    let peer_connection = Arc::new(
        api.new_peer_connection(RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        })
        .await
        .map_err(webrtc_api_error)?,
    );

    if transport == WebRtcTransport::Media {
        let track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_VP8.to_string(),
                ..Default::default()
            },
            "appmon-video".to_string(),
            "appmon-preview".to_string(),
        ));
        let _sender = peer_connection
            .add_track(Arc::clone(&track) as Arc<_>)
            .await
            .map_err(webrtc_api_error)?;
        tokio::spawn(send_webrtc_video_preview(
            state.controller.clone(),
            id.clone(),
            track,
            Arc::clone(&peer_connection),
            WebRtcStreamSettings {
                fps,
                stream_format: stream_format.clone(),
                max_width,
                quality,
            },
        ));
    } else {
        let controller = state.controller.clone();
        let peer_for_channel = Arc::clone(&peer_connection);
        peer_connection.on_data_channel(Box::new(move |data_channel| {
            let controller = controller.clone();
            let id = id.clone();
            let peer_connection = Arc::clone(&peer_for_channel);
            let stream_format = stream_format.clone();
            Box::pin(async move {
                if data_channel.label() == "appmon-preview" {
                    tokio::spawn(send_webrtc_preview(
                        controller,
                        id,
                        data_channel,
                        peer_connection,
                        WebRtcStreamSettings {
                            fps,
                            stream_format,
                            max_width,
                            quality,
                        },
                    ));
                }
            })
        }));
    }

    let sessions = Arc::clone(&state.webrtc_sessions);
    peer_connection.on_peer_connection_state_change(Box::new(move |connection_state| {
        let sessions = Arc::clone(&sessions);
        Box::pin(async move {
            if matches!(
                connection_state,
                RTCPeerConnectionState::Failed
                    | RTCPeerConnectionState::Closed
                    | RTCPeerConnectionState::Disconnected
            ) {
                sessions.lock().await.remove(&session_id);
            }
        })
    }));

    let mut gather_complete = peer_connection.gathering_complete_promise().await;
    peer_connection
        .set_remote_description(req.offer)
        .await
        .map_err(webrtc_api_error)?;
    let answer = peer_connection
        .create_answer(None)
        .await
        .map_err(webrtc_api_error)?;
    peer_connection
        .set_local_description(answer)
        .await
        .map_err(webrtc_api_error)?;
    let _ = gather_complete.recv().await;
    let answer = peer_connection.local_description().await.ok_or_else(|| {
        ApiError(AppError::InvalidInput(
            "WebRTC answer was not created".into(),
        ))
    })?;

    state
        .webrtc_sessions
        .lock()
        .await
        .insert(session_id, peer_connection);

    Ok(Json(WebRtcOfferResponse { session_id, answer }))
}

#[derive(Clone)]
struct WebRtcStreamSettings {
    fps: u32,
    stream_format: String,
    max_width: u32,
    quality: u8,
}

async fn send_webrtc_video_preview<R: ProcessRunner + Clone>(
    controller: DeviceManager<R>,
    id: DeviceId,
    track: Arc<TrackLocalStaticSample>,
    peer_connection: Arc<RTCPeerConnection>,
    settings: WebRtcStreamSettings,
) {
    if let Err(error) = run_vp8_encoder_preview(
        controller,
        id.clone(),
        track,
        Arc::clone(&peer_connection),
        settings,
    )
    .await
    {
        warn!(device = %id.web_id(), %error, "WebRTC video preview stopped");
    }
    let _ = peer_connection.close().await;
}

async fn run_vp8_encoder_preview<R: ProcessRunner + Clone>(
    controller: DeviceManager<R>,
    id: DeviceId,
    track: Arc<TrackLocalStaticSample>,
    peer_connection: Arc<RTCPeerConnection>,
    settings: WebRtcStreamSettings,
) -> Result<(), String> {
    let frame_delay = Duration::from_millis(1_000 / u64::from(settings.fps));
    let connect_started = Instant::now();
    loop {
        match peer_connection.connection_state() {
            RTCPeerConnectionState::Connected => break,
            RTCPeerConnectionState::Failed
            | RTCPeerConnectionState::Closed
            | RTCPeerConnectionState::Disconnected => {
                return Err("WebRTC video connection closed before encoder start".to_string())
            }
            _ if connect_started.elapsed() > Duration::from_secs(8) => {
                return Err("WebRTC video connection timed out".to_string())
            }
            _ => tokio::time::sleep(Duration::from_millis(20)).await,
        }
    }

    let mut child = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-fflags",
            "nobuffer",
            "-flags",
            "low_delay",
            "-f",
            "mjpeg",
            "-framerate",
            &settings.fps.to_string(),
            "-i",
            "pipe:0",
            "-an",
            "-c:v",
            "libvpx",
            "-deadline",
            "realtime",
            "-cpu-used",
            "8",
            "-lag-in-frames",
            "0",
            "-auto-alt-ref",
            "0",
            "-error-resilient",
            "1",
            "-g",
            "30",
            "-quality",
            "realtime",
            "-f",
            "ivf",
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start ffmpeg: {error}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "ffmpeg stdin is unavailable".to_string())?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "ffmpeg stdout is unavailable".to_string())?;

    let writer_id = id.clone();
    let writer_peer = Arc::clone(&peer_connection);
    let writer_settings = settings.clone();
    let writer = async move {
        loop {
            if matches!(
                writer_peer.connection_state(),
                RTCPeerConnectionState::Failed
                    | RTCPeerConnectionState::Closed
                    | RTCPeerConnectionState::Disconnected
            ) {
                break;
            }
            let frame_started = Instant::now();
            let screenshot = controller
                .screenshot(&writer_id)
                .await
                .map_err(|error| error.to_string())?;
            let jpeg = prepare_video_jpeg_frame(
                screenshot.content_type,
                screenshot.bytes,
                writer_settings.max_width,
                writer_settings.quality,
            )
            .map_err(|error| error.to_string())?;
            stdin
                .write_all(&jpeg)
                .await
                .map_err(|error| format!("ffmpeg stdin write failed: {error}"))?;
            stdin
                .flush()
                .await
                .map_err(|error| format!("ffmpeg stdin flush failed: {error}"))?;
            let elapsed = frame_started.elapsed();
            if elapsed < frame_delay {
                tokio::time::sleep(frame_delay - elapsed).await;
            }
        }
        Ok::<(), String>(())
    };

    let reader = async move {
        let mut file_header = [0_u8; 32];
        stdout
            .read_exact(&mut file_header)
            .await
            .map_err(|error| format!("ffmpeg ivf header read failed: {error}"))?;
        loop {
            let mut frame_header = [0_u8; 12];
            stdout
                .read_exact(&mut frame_header)
                .await
                .map_err(|error| format!("ffmpeg ivf frame header read failed: {error}"))?;
            let frame_len = u32::from_le_bytes([
                frame_header[0],
                frame_header[1],
                frame_header[2],
                frame_header[3],
            ]) as usize;
            if frame_len == 0 || frame_len > 8 * 1024 * 1024 {
                return Err(format!(
                    "ffmpeg emitted invalid VP8 frame length: {frame_len}"
                ));
            }
            let mut frame = vec![0_u8; frame_len];
            stdout
                .read_exact(&mut frame)
                .await
                .map_err(|error| format!("ffmpeg ivf frame read failed: {error}"))?;
            track
                .write_sample(&Sample {
                    data: Bytes::from(frame),
                    duration: frame_delay,
                    ..Default::default()
                })
                .await
                .map_err(|error| format!("WebRTC video sample write failed: {error}"))?;
        }
    };

    tokio::select! {
        result = writer => result?,
        result = reader => result?,
    }

    let _ = child.kill().await;
    Ok(())
}

fn prepare_video_jpeg_frame(
    content_type: &str,
    bytes: Vec<u8>,
    max_width: u32,
    quality: u8,
) -> Result<Vec<u8>, image::ImageError> {
    if content_type == "image/jpeg" && max_width >= 4096 {
        Ok(bytes)
    } else {
        encode_stream_frame(&bytes, max_width, quality).map(|(_, bytes)| bytes)
    }
}

async fn send_webrtc_preview<R: ProcessRunner + Clone>(
    controller: DeviceManager<R>,
    id: DeviceId,
    data_channel: Arc<RTCDataChannel>,
    peer_connection: Arc<RTCPeerConnection>,
    settings: WebRtcStreamSettings,
) {
    let frame_delay = Duration::from_millis(1_000 / u64::from(settings.fps));
    let mut frame_seq = 0_u32;

    loop {
        if data_channel.ready_state() == RTCDataChannelState::Open {
            break;
        }
        if matches!(
            data_channel.ready_state(),
            RTCDataChannelState::Closing | RTCDataChannelState::Closed
        ) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    loop {
        let frame_started = Instant::now();
        if matches!(
            data_channel.ready_state(),
            RTCDataChannelState::Closing | RTCDataChannelState::Closed
        ) {
            break;
        }

        match controller.screenshot(&id).await {
            Ok(screenshot) => {
                let encode_jpeg = match settings.stream_format.as_str() {
                    "jpeg" => true,
                    "native" => false,
                    _ => id.kind != DeviceKind::Android && screenshot.content_type != "image/jpeg",
                };
                let frame = if encode_jpeg {
                    encode_stream_frame(&screenshot.bytes, settings.max_width, settings.quality)
                        .unwrap_or_else(|_| (screenshot.content_type, screenshot.bytes))
                } else {
                    (screenshot.content_type, screenshot.bytes)
                };

                frame_seq = frame_seq.wrapping_add(1);
                if let Err(error) =
                    send_webrtc_frame(&data_channel, frame_seq, frame.0, &frame.1).await
                {
                    warn!(device = %id.web_id(), %error, "WebRTC preview send failed");
                    break;
                }
            }
            Err(error) => {
                warn!(device = %id.web_id(), %error, "WebRTC preview stopped");
                break;
            }
        }

        let elapsed = frame_started.elapsed();
        if elapsed < frame_delay {
            tokio::time::sleep(frame_delay - elapsed).await;
        }
    }

    let _ = peer_connection.close().await;
}

async fn send_webrtc_frame(
    data_channel: &RTCDataChannel,
    frame_seq: u32,
    content_type: &str,
    frame: &[u8],
) -> Result<(), webrtc::Error> {
    const CHUNK_SIZE: usize = 12 * 1024;
    let content_type = content_type.as_bytes();
    let mut offset = 0_usize;
    while offset < frame.len() {
        while data_channel.buffered_amount().await > 4 * 1024 * 1024 {
            tokio::time::sleep(Duration::from_millis(8)).await;
        }

        let end = (offset + CHUNK_SIZE).min(frame.len());
        let first = offset == 0;
        let last = end == frame.len();
        let content_type_len = if first {
            content_type.len().min(u8::MAX as usize)
        } else {
            0
        };
        let mut packet = Vec::with_capacity(10 + content_type_len + (end - offset));
        packet.extend_from_slice(&frame_seq.to_be_bytes());
        packet.push((if first { 1 } else { 0 }) | (if last { 2 } else { 0 }));
        packet.push(content_type_len as u8);
        packet.extend_from_slice(&(frame.len() as u32).to_be_bytes());
        packet.extend_from_slice(&content_type[..content_type_len]);
        packet.extend_from_slice(&frame[offset..end]);
        data_channel.send(&Bytes::from(packet)).await?;
        offset = end;
    }
    Ok(())
}

fn webrtc_api_error(error: webrtc::Error) -> ApiError {
    ApiError(AppError::InvalidInput(format!(
        "WebRTC negotiation failed: {error}"
    )))
}

async fn emulator_webrtc_ws<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let id = DeviceId::parse(&id)?;
    if id.kind != DeviceKind::Android {
        return Err(ApiError(AppError::UnsupportedCapability(
            "native emulator WebRTC is only available for Android emulators".to_string(),
        )));
    }
    let endpoint = state
        .controller
        .android_emulator_grpc_endpoint()
        .ok_or_else(|| {
            ApiError(AppError::UnsupportedCapability(
                "set APPMON_ANDROID_GRPC_ENDPOINT to use native emulator WebRTC".to_string(),
            ))
        })?
        .to_string();
    Ok(ws.on_upgrade(move |socket| handle_emulator_webrtc_socket(socket, endpoint, id)))
}

async fn handle_emulator_webrtc_socket(mut socket: WebSocket, endpoint: String, id: DeviceId) {
    let rtc_id = match android_grpc::request_rtc_stream(&endpoint).await {
        Ok(rtc_id) => rtc_id,
        Err(error) => {
            let _ = socket
                .send(emulator_webrtc_error_message(format!(
                    "failed to start native emulator WebRTC: {error}"
                )))
                .await;
            let _ = socket.close().await;
            return;
        }
    };

    let mut messages = match android_grpc::receive_jsep_messages(&endpoint, rtc_id.clone()).await {
        Ok(messages) => messages,
        Err(error) => {
            let _ = socket
                .send(emulator_webrtc_error_message(format!(
                    "failed to read native emulator WebRTC messages: {error}"
                )))
                .await;
            let _ = socket.close().await;
            return;
        }
    };

    loop {
        tokio::select! {
            message = messages.message() => {
                match message {
                    Ok(Some(message)) => {
                        if socket.send(Message::Text(message.message)).await.is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(error) => {
                        warn!(device = %id.web_id(), %error, "native emulator WebRTC message stream stopped");
                        break;
                    }
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(message))) => {
                        if let Err(error) = android_grpc::send_jsep_message(&endpoint, rtc_id.clone(), message).await {
                            warn!(device = %id.web_id(), %error, "failed to forward browser JSEP to emulator");
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(error)) => {
                        warn!(device = %id.web_id(), %error, "native emulator WebRTC browser socket stopped");
                        break;
                    }
                }
            }
        }
    }
}

fn emulator_webrtc_error_message(error: String) -> Message {
    Message::Text(serde_json::json!({ "error": error }).to_string())
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

async fn motion<R: ProcessRunner + Clone>(
    State(state): State<AppState<R>>,
    Path(id): Path<String>,
    Json(req): Json<MotionRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let id = DeviceId::parse(&id)?;
    state.controller.motion(&id, req).await?;
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
        .send(Message::Text("Appmon WebSocket connected".to_string()))
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
        ControlCommand::Motion {
            action,
            x,
            y,
            source_width,
            source_height,
        } => {
            state
                .controller
                .motion(
                    &id,
                    MotionRequest {
                        action,
                        x,
                        y,
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
    info!(%bind, "appmon udp control listening");
    let state = AppState {
        controller,
        webrtc_sessions: Arc::new(Mutex::new(HashMap::new())),
    };
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
    Motion {
        action: MotionAction,
        x: u32,
        y: u32,
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

    use appmon_core::{AppConfig, AppResult, CommandOutput};
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
            emulator_path: "/bin/echo".into(),
            android_grpc_endpoint: None,
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
