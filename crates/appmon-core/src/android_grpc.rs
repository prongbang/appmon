use std::time::Duration;

use tonic::transport::Channel;

use crate::{
    AppError, AppResult, KeyRequest, MotionAction, MotionRequest, TapRequest, TextRequest,
};

pub mod proto {
    tonic::include_proto!("android.emulation.control");
}

use proto::{
    emulator_controller_client::EmulatorControllerClient, keyboard_event::KeyEventType,
    touch::EventExpiration, KeyboardEvent, Touch, TouchEvent,
};

const TOUCH_IDENTIFIER: i32 = 1;
const TOUCH_PRESSURE: i32 = 0x7000;
const TOUCH_SIZE: i32 = 8;

pub async fn send_tap(endpoint: &str, req: &TapRequest) -> AppResult<()> {
    send_touch(
        endpoint,
        req.x,
        req.y,
        TOUCH_PRESSURE,
        EventExpiration::NeverExpire,
    )
    .await?;
    send_touch(endpoint, req.x, req.y, 0, EventExpiration::Unspecified).await
}

pub async fn send_motion(endpoint: &str, req: &MotionRequest) -> AppResult<()> {
    let pressure = match req.action {
        MotionAction::Down | MotionAction::Move => TOUCH_PRESSURE,
        MotionAction::Up => 0,
    };
    let expiration = match req.action {
        MotionAction::Down | MotionAction::Move => EventExpiration::NeverExpire,
        MotionAction::Up => EventExpiration::Unspecified,
    };
    send_touch(endpoint, req.x, req.y, pressure, expiration).await
}

pub async fn send_key(endpoint: &str, req: &KeyRequest) -> AppResult<()> {
    let mut client = connect(endpoint).await?;
    client
        .send_key(KeyboardEvent {
            event_type: KeyEventType::Keypress.into(),
            key: android_grpc_key(&req.key).to_string(),
            ..KeyboardEvent::default()
        })
        .await
        .map_err(grpc_error)?;
    Ok(())
}

pub async fn send_text(endpoint: &str, req: &TextRequest) -> AppResult<()> {
    let mut client = connect(endpoint).await?;
    client
        .send_key(KeyboardEvent {
            text: req.text.clone(),
            ..KeyboardEvent::default()
        })
        .await
        .map_err(grpc_error)?;
    Ok(())
}

async fn send_touch(
    endpoint: &str,
    x: u32,
    y: u32,
    pressure: i32,
    expiration: EventExpiration,
) -> AppResult<()> {
    let mut client = connect(endpoint).await?;
    client
        .send_touch(TouchEvent {
            touches: vec![Touch {
                x: x.min(i32::MAX as u32) as i32,
                y: y.min(i32::MAX as u32) as i32,
                identifier: TOUCH_IDENTIFIER,
                pressure,
                touch_major: TOUCH_SIZE,
                touch_minor: TOUCH_SIZE,
                expiration: expiration.into(),
            }],
            display: 0,
        })
        .await
        .map_err(grpc_error)?;
    Ok(())
}

async fn connect(endpoint: &str) -> AppResult<EmulatorControllerClient<Channel>> {
    let channel = Channel::from_shared(endpoint.to_string())
        .map_err(|error| AppError::InvalidInput(format!("invalid Android gRPC endpoint: {error}")))?
        .connect_timeout(Duration::from_millis(120))
        .timeout(Duration::from_millis(250))
        .connect()
        .await
        .map_err(grpc_error)?;
    Ok(EmulatorControllerClient::new(channel))
}

fn grpc_error(error: impl std::fmt::Display) -> AppError {
    AppError::UnsupportedCapability(format!("Android emulator gRPC unavailable: {error}"))
}

fn android_grpc_key(key: &str) -> &str {
    match key.to_ascii_uppercase().as_str() {
        "BACK" => "GoBack",
        "HOME" => "GoHome",
        "APP_SWITCH" | "RECENTS" => "AppSwitch",
        "POWER" => "Power",
        _ => key,
    }
}
