use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    process::Stdio,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tokio::{
    io::AsyncWriteExt,
    process::{ChildStdin, Command},
    sync::Mutex,
};
use tracing::info;
use uuid::Uuid;

use crate::{
    android_grpc, validation, AppConfig, AppError, AppResult, CommandOutput, ProcessRunner,
    TokioProcessRunner,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    Android,
    Ios,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceId {
    pub kind: DeviceKind,
    pub raw: String,
}

impl DeviceId {
    pub fn parse(value: &str) -> AppResult<Self> {
        let (prefix, raw) = value.split_once(':').ok_or_else(|| {
            AppError::InvalidInput("device id must be android:<id> or ios:<id>".to_string())
        })?;
        validation::non_empty(raw, "device id")?;
        let kind = match prefix {
            "android" => DeviceKind::Android,
            "ios" => DeviceKind::Ios,
            _ => {
                return Err(AppError::InvalidInput(
                    "unknown device platform".to_string(),
                ))
            }
        };
        Ok(Self {
            kind,
            raw: raw.to_string(),
        })
    }

    pub fn web_id(&self) -> String {
        match self.kind {
            DeviceKind::Android => format!("android:{}", self.raw),
            DeviceKind::Ios => format!("ios:{}", self.raw),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub kind: DeviceKind,
    pub name: String,
    pub state: String,
    pub details: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Screenshot {
    pub content_type: &'static str,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TapRequest {
    pub x: u32,
    pub y: u32,
    pub source_width: Option<u32>,
    pub source_height: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SwipeRequest {
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
    pub duration_ms: Option<u32>,
    pub source_width: Option<u32>,
    pub source_height: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionAction {
    Down,
    Move,
    Up,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MotionRequest {
    pub action: MotionAction,
    pub x: u32,
    pub y: u32,
    pub source_width: Option<u32>,
    pub source_height: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TextRequest {
    pub text: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct KeyRequest {
    pub key: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppInstallRequest {
    pub path: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppLaunchRequest {
    pub app_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppTerminateRequest {
    pub app_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LogRequest {
    pub lines: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RecordRequest {
    pub output_path: Option<PathBuf>,
}

#[derive(Clone)]
pub struct DeviceManager<R: ProcessRunner = TokioProcessRunner> {
    config: AppConfig,
    runner: Arc<R>,
    recordings: Arc<Mutex<HashMap<String, RecordingJob>>>,
    android_inputs: Arc<Mutex<HashMap<String, AndroidInputSession>>>,
    ios_points: Arc<Mutex<HashMap<String, PointDimensions>>>,
    ios_companions: Arc<Mutex<HashSet<String>>>,
}

pub type AppController = DeviceManager<TokioProcessRunner>;

impl DeviceManager<TokioProcessRunner> {
    pub fn new(config: AppConfig) -> Self {
        Self::with_runner(config, TokioProcessRunner)
    }
}

impl<R: ProcessRunner> DeviceManager<R> {
    pub fn with_runner(config: AppConfig, runner: R) -> Self {
        Self {
            config,
            runner: Arc::new(runner),
            recordings: Arc::new(Mutex::new(HashMap::new())),
            android_inputs: Arc::new(Mutex::new(HashMap::new())),
            ios_points: Arc::new(Mutex::new(HashMap::new())),
            ios_companions: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn list_devices(&self) -> AppResult<Vec<Device>> {
        let mut devices = Vec::new();
        devices.extend(self.list_android_devices().await.unwrap_or_default());
        devices.extend(self.list_ios_devices().await.unwrap_or_default());
        Ok(devices)
    }

    pub fn android_emulator_grpc_endpoint(&self) -> Option<&str> {
        self.config.android_grpc_endpoint.as_deref()
    }

    pub async fn screenshot(&self, id: &DeviceId) -> AppResult<Screenshot> {
        match id.kind {
            DeviceKind::Android => {
                let args = vec![
                    "-s".to_string(),
                    id.raw.clone(),
                    "exec-out".to_string(),
                    "screencap".to_string(),
                    "-p".to_string(),
                ];
                Ok(Screenshot {
                    content_type: "image/png",
                    bytes: self
                        .run_logged("android_screenshot", &self.config.adb_path, &args)
                        .await?
                        .stdout,
                })
            }
            DeviceKind::Ios => {
                let args = vec![
                    "simctl".to_string(),
                    "io".to_string(),
                    id.raw.clone(),
                    "screenshot".to_string(),
                    "--type=jpeg".to_string(),
                    "-".to_string(),
                ];
                Ok(Screenshot {
                    content_type: "image/jpeg",
                    bytes: self
                        .run_logged("ios_screenshot_stdout", &self.config.xcrun_path, &args)
                        .await?
                        .stdout,
                })
            }
        }
    }

    pub async fn tap(&self, id: &DeviceId, req: TapRequest) -> AppResult<()> {
        validation::coordinate(req.x, "x")?;
        validation::coordinate(req.y, "y")?;
        match id.kind {
            DeviceKind::Android => {
                if let Some(endpoint) = self.android_grpc_endpoint() {
                    match android_grpc::send_tap(endpoint, &req).await {
                        Ok(()) => return Ok(()),
                        Err(error) => {
                            info!(device = %id.raw, %error, "Android gRPC tap unavailable, falling back to adb input");
                        }
                    }
                }
                self.run_android_input(
                    id,
                    format!("input tap {} {}", req.x, req.y),
                    "android_tap_fast",
                )
                .await?;
            }
            DeviceKind::Ios => {
                if let Err(error) = self.run_ios_idb_tap(id, &req).await {
                    info!(device = %id.raw, %error, "idb tap unavailable, falling back to simulator window tap");
                    self.run_ios_window_tap(req).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn swipe(&self, id: &DeviceId, req: SwipeRequest) -> AppResult<()> {
        for (name, value) in [
            ("x1", req.x1),
            ("y1", req.y1),
            ("x2", req.x2),
            ("y2", req.y2),
        ] {
            validation::coordinate(value, name)?;
        }
        let duration = req.duration_ms.unwrap_or(300).min(60_000);
        match id.kind {
            DeviceKind::Android => {
                self.run_android_input(
                    id,
                    format!(
                        "input swipe {} {} {} {} {}",
                        req.x1, req.y1, req.x2, req.y2, duration
                    ),
                    "android_swipe_fast",
                )
                .await?;
            }
            DeviceKind::Ios => {
                self.run_ios_idb_swipe(id, &req, duration).await?;
            }
        }
        Ok(())
    }

    pub async fn motion(&self, id: &DeviceId, req: MotionRequest) -> AppResult<()> {
        validation::coordinate(req.x, "x")?;
        validation::coordinate(req.y, "y")?;
        match id.kind {
            DeviceKind::Android => {
                if let Some(endpoint) = self.android_grpc_endpoint() {
                    match android_grpc::send_motion(endpoint, &req).await {
                        Ok(()) => return Ok(()),
                        Err(error) => {
                            info!(device = %id.raw, %error, "Android gRPC motion unavailable, falling back to adb input");
                        }
                    }
                }
                self.run_android_input(
                    id,
                    format!(
                        "input motionevent {} {} {}",
                        android_motion_action(req.action),
                        req.x,
                        req.y
                    ),
                    "android_motion_fast",
                )
                .await?;
            }
            DeviceKind::Ios => match req.action {
                MotionAction::Down | MotionAction::Move => {}
                MotionAction::Up => {
                    self.run_ios_idb_tap(
                        id,
                        &TapRequest {
                            x: req.x,
                            y: req.y,
                            source_width: req.source_width,
                            source_height: req.source_height,
                        },
                    )
                    .await?;
                }
            },
        }
        Ok(())
    }

    pub async fn text(&self, id: &DeviceId, req: TextRequest) -> AppResult<()> {
        validation::text_input(&req.text)?;
        match id.kind {
            DeviceKind::Android => {
                if let Some(endpoint) = self.android_grpc_endpoint() {
                    match android_grpc::send_text(endpoint, &req).await {
                        Ok(()) => return Ok(()),
                        Err(error) => {
                            info!(device = %id.raw, %error, "Android gRPC text unavailable, falling back to adb input");
                        }
                    }
                }
                let escaped = req.text.replace(' ', "%s");
                self.run_android_input(
                    id,
                    format!("input text {}", shell_quote(&escaped)),
                    "android_text_fast",
                )
                .await?;
            }
            DeviceKind::Ios => {
                self.run_ios_idb_text(id, &req.text).await?;
            }
        }
        Ok(())
    }

    pub async fn key(&self, id: &DeviceId, req: KeyRequest) -> AppResult<()> {
        validation::key_input(&req.key)?;
        match id.kind {
            DeviceKind::Android => {
                if let Some(endpoint) = self.android_grpc_endpoint() {
                    match android_grpc::send_key(endpoint, &req).await {
                        Ok(()) => return Ok(()),
                        Err(error) => {
                            info!(device = %id.raw, %error, "Android gRPC key unavailable, falling back to adb input");
                        }
                    }
                }
                self.run_android_input(
                    id,
                    format!("input keyevent {}", shell_quote(&req.key)),
                    "android_key_fast",
                )
                .await?;
            }
            DeviceKind::Ios => {
                self.run_ios_idb_key(id, &req.key).await?;
            }
        }
        Ok(())
    }

    pub async fn start_device(&self, id: &DeviceId) -> AppResult<()> {
        match id.kind {
            DeviceKind::Android => {
                let mut args = vec!["-avd".to_string(), id.raw.clone()];
                if let Some(port) = self
                    .android_grpc_endpoint()
                    .and_then(android_grpc_port_from_endpoint)
                {
                    args.extend(["-grpc".to_string(), port.to_string()]);
                }
                Command::new(&self.config.emulator_path)
                    .args(args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()?;
                info!(device = %id.raw, "android emulator start requested");
            }
            DeviceKind::Ios => {
                let args = vec!["simctl".to_string(), "boot".to_string(), id.raw.clone()];
                match self
                    .run_logged("ios_boot", &self.config.xcrun_path, &args)
                    .await
                {
                    Err(AppError::CommandFailed { stderr, .. })
                        if stderr.contains("current state: Booted")
                            || stderr
                                .contains("Unable to boot device in current state: Booted") =>
                    {
                        info!(device = %id.raw, "iOS simulator already booted");
                    }
                    other => {
                        other?;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn stop_device(&self, id: &DeviceId) -> AppResult<()> {
        match id.kind {
            DeviceKind::Android => {
                let args = vec![
                    "-s".to_string(),
                    id.raw.clone(),
                    "emu".to_string(),
                    "kill".to_string(),
                ];
                self.run_logged("android_emulator_stop", &self.config.adb_path, &args)
                    .await?;
                self.android_inputs.lock().await.remove(&id.raw);
            }
            DeviceKind::Ios => {
                let args = vec!["simctl".to_string(), "shutdown".to_string(), id.raw.clone()];
                match self
                    .run_logged("ios_shutdown", &self.config.xcrun_path, &args)
                    .await
                {
                    Err(AppError::CommandFailed { stderr, .. })
                        if stderr.contains("current state: Shutdown")
                            || stderr.contains(
                                "Unable to shutdown device in current state: Shutdown",
                            ) =>
                    {
                        info!(device = %id.raw, "iOS simulator already shutdown");
                    }
                    other => {
                        other?;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn install(&self, id: &DeviceId, req: AppInstallRequest) -> AppResult<()> {
        validation::readable_file(&req.path)?;
        match id.kind {
            DeviceKind::Android => {
                let args = vec![
                    "-s".to_string(),
                    id.raw.clone(),
                    "install".to_string(),
                    "-r".to_string(),
                    req.path.display().to_string(),
                ];
                self.run_logged("android_install", &self.config.adb_path, &args)
                    .await?;
            }
            DeviceKind::Ios => {
                let args = vec![
                    "simctl".to_string(),
                    "install".to_string(),
                    id.raw.clone(),
                    req.path.display().to_string(),
                ];
                self.run_logged("ios_install", &self.config.xcrun_path, &args)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn launch(&self, id: &DeviceId, req: AppLaunchRequest) -> AppResult<()> {
        validation::package_or_bundle_id(&req.app_id)?;
        match id.kind {
            DeviceKind::Android => {
                let args = vec![
                    "-s".to_string(),
                    id.raw.clone(),
                    "shell".to_string(),
                    "monkey".to_string(),
                    "-p".to_string(),
                    req.app_id,
                    "1".to_string(),
                ];
                self.run_logged("android_launch", &self.config.adb_path, &args)
                    .await?;
            }
            DeviceKind::Ios => {
                let args = vec![
                    "simctl".to_string(),
                    "launch".to_string(),
                    id.raw.clone(),
                    req.app_id,
                ];
                self.run_logged("ios_launch", &self.config.xcrun_path, &args)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn terminate(&self, id: &DeviceId, req: AppTerminateRequest) -> AppResult<()> {
        validation::package_or_bundle_id(&req.app_id)?;
        match id.kind {
            DeviceKind::Android => {
                let args = vec![
                    "-s".to_string(),
                    id.raw.clone(),
                    "shell".to_string(),
                    "am".to_string(),
                    "force-stop".to_string(),
                    req.app_id,
                ];
                self.run_logged("android_terminate", &self.config.adb_path, &args)
                    .await?;
            }
            DeviceKind::Ios => {
                let args = vec![
                    "simctl".to_string(),
                    "terminate".to_string(),
                    id.raw.clone(),
                    req.app_id,
                ];
                self.run_logged("ios_terminate", &self.config.xcrun_path, &args)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn logs(&self, id: &DeviceId, req: LogRequest) -> AppResult<String> {
        let lines = req.lines.unwrap_or(200).clamp(1, 2000).to_string();
        let output = match id.kind {
            DeviceKind::Android => {
                let args = vec![
                    "-s".to_string(),
                    id.raw.clone(),
                    "logcat".to_string(),
                    "-d".to_string(),
                    "-t".to_string(),
                    lines,
                ];
                self.run_logged("android_logs", &self.config.adb_path, &args)
                    .await?
            }
            DeviceKind::Ios => {
                let args = vec![
                    "simctl".to_string(),
                    "spawn".to_string(),
                    id.raw.clone(),
                    "log".to_string(),
                    "show".to_string(),
                    "--style".to_string(),
                    "compact".to_string(),
                    "--last".to_string(),
                    "2m".to_string(),
                ];
                self.run_logged("ios_logs", &self.config.xcrun_path, &args)
                    .await?
            }
        };
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn record_start(&self, id: &DeviceId, req: RecordRequest) -> AppResult<PathBuf> {
        let key = id.web_id();
        if self.recordings.lock().await.contains_key(&key) {
            return Err(AppError::InvalidInput(format!(
                "recording is already running for {key}"
            )));
        }

        match id.kind {
            DeviceKind::Android => {
                let remote_path = PathBuf::from(format!("/sdcard/appmon-{}.mp4", Uuid::new_v4()));
                let args = vec![
                    "-s".to_string(),
                    id.raw.clone(),
                    "shell".to_string(),
                    "screenrecord".to_string(),
                    remote_path.display().to_string(),
                ];
                let child = spawn_recording(&self.config.adb_path, &args)?;
                self.recordings.lock().await.insert(
                    key,
                    RecordingJob {
                        output_path: remote_path.clone(),
                        child,
                    },
                );
                Ok(remote_path)
            }
            DeviceKind::Ios => {
                let path = req.output_path.unwrap_or_else(|| {
                    std::env::temp_dir().join(format!("appmon-ios-{}.mp4", Uuid::new_v4()))
                });
                let args = vec![
                    "simctl".to_string(),
                    "io".to_string(),
                    id.raw.clone(),
                    "recordVideo".to_string(),
                    path.display().to_string(),
                ];
                let child = spawn_recording(&self.config.xcrun_path, &args)?;
                self.recordings.lock().await.insert(
                    key,
                    RecordingJob {
                        output_path: path.clone(),
                        child,
                    },
                );
                Ok(path)
            }
        }
    }

    pub async fn record_stop(&self, id: &DeviceId) -> AppResult<PathBuf> {
        let key = id.web_id();
        let Some(mut job) = self.recordings.lock().await.remove(&key) else {
            return Err(AppError::InvalidInput(format!(
                "no recording is running for {key}"
            )));
        };
        job.child.kill().await?;
        let _ = job.child.wait().await;
        Ok(job.output_path)
    }

    async fn list_android_devices(&self) -> AppResult<Vec<Device>> {
        let args = vec!["devices".to_string(), "-l".to_string()];
        let output = self
            .run_logged("android_list", &self.config.adb_path, &args)
            .await?;
        let mut devices = parse_adb_devices(&String::from_utf8_lossy(&output.stdout));
        let avd_args = vec!["-list-avds".to_string()];
        if let Ok(output) = self
            .run_logged("android_avd_list", &self.config.emulator_path, &avd_args)
            .await
        {
            devices.extend(parse_android_avds(&String::from_utf8_lossy(&output.stdout)));
        }
        Ok(devices)
    }

    async fn list_ios_devices(&self) -> AppResult<Vec<Device>> {
        let args = vec![
            "simctl".to_string(),
            "list".to_string(),
            "devices".to_string(),
            "--json".to_string(),
        ];
        let output = self
            .run_logged("ios_list", &self.config.xcrun_path, &args)
            .await?;
        parse_simctl_devices(&output.stdout)
    }

    async fn run_android_input(
        &self,
        id: &DeviceId,
        command: String,
        op: &'static str,
    ) -> AppResult<()> {
        let start = Instant::now();
        let write_result = self.write_android_input(id, &command).await;
        let result = match write_result {
            Ok(()) => Ok(()),
            Err(error) => {
                self.android_inputs.lock().await.remove(&id.raw);
                Err(error)
            }
        };
        info!(
            op,
            device = %id.raw,
            duration_ms = start.elapsed().as_millis(),
            success = result.is_ok(),
            "android input command queued"
        );
        result
    }

    fn android_grpc_endpoint(&self) -> Option<&str> {
        self.android_emulator_grpc_endpoint()
    }

    async fn write_android_input(&self, id: &DeviceId, command: &str) -> AppResult<()> {
        let mut inputs = self.android_inputs.lock().await;
        if !inputs.contains_key(&id.raw) {
            inputs.insert(
                id.raw.clone(),
                AndroidInputSession::spawn(&self.config.adb_path, &id.raw)?,
            );
        }
        let session = inputs
            .get_mut(&id.raw)
            .expect("android input session is inserted before use");
        session.write_command(command).await
    }

    async fn run_ios_idb_tap(&self, id: &DeviceId, req: &TapRequest) -> AppResult<()> {
        let (x, y) = self
            .map_ios_point(id, req.x, req.y, req.source_width, req.source_height)
            .await?;
        self.run_idb_ui(
            id,
            "ios_idb_tap",
            vec!["ui".into(), "tap".into(), x.to_string(), y.to_string()],
            Duration::from_secs(3),
        )
        .await
    }

    async fn run_ios_idb_swipe(
        &self,
        id: &DeviceId,
        req: &SwipeRequest,
        duration_ms: u32,
    ) -> AppResult<()> {
        let (x1, y1) = self
            .map_ios_point(id, req.x1, req.y1, req.source_width, req.source_height)
            .await?;
        let (x2, y2) = self
            .map_ios_point(id, req.x2, req.y2, req.source_width, req.source_height)
            .await?;
        let duration = format!("{:.3}", f64::from(duration_ms) / 1000.0);
        self.run_idb_ui(
            id,
            "ios_idb_swipe",
            vec![
                "ui".into(),
                "swipe".into(),
                x1.to_string(),
                y1.to_string(),
                x2.to_string(),
                y2.to_string(),
                "--duration".into(),
                duration,
            ],
            Duration::from_secs(5),
        )
        .await
    }

    async fn run_ios_idb_text(&self, id: &DeviceId, text: &str) -> AppResult<()> {
        self.run_idb_ui(
            id,
            "ios_idb_text",
            vec!["ui".into(), "text".into(), text.to_string()],
            Duration::from_secs(5),
        )
        .await
    }

    async fn run_ios_idb_key(&self, id: &DeviceId, key: &str) -> AppResult<()> {
        if is_ios_app_switch_key(key) {
            return self.run_ios_idb_app_switch(id).await;
        }
        if let Some(button) = ios_button_for_key(key) {
            return self
                .run_idb_ui(
                    id,
                    "ios_idb_button",
                    vec!["ui".into(), "button".into(), button.to_string()],
                    Duration::from_secs(3),
                )
                .await;
        }
        self.run_idb_ui(
            id,
            "ios_idb_key",
            vec!["ui".into(), "key".into(), key.to_string()],
            Duration::from_secs(5),
        )
        .await
    }

    async fn run_ios_idb_app_switch(&self, id: &DeviceId) -> AppResult<()> {
        let points = self.ios_point_dimensions(id).await?;
        let x = points.width / 2;
        let y_start = points.height.saturating_sub(8);
        let y_end = points.height.saturating_mul(52) / 100;
        self.run_idb_ui(
            id,
            "ios_idb_app_switch",
            vec![
                "ui".into(),
                "swipe".into(),
                x.to_string(),
                y_start.to_string(),
                x.to_string(),
                y_end.to_string(),
                "--duration".into(),
                "0.650".into(),
            ],
            Duration::from_secs(5),
        )
        .await
    }

    async fn run_idb_ui(
        &self,
        id: &DeviceId,
        op: &'static str,
        args: Vec<String>,
        timeout: Duration,
    ) -> AppResult<()> {
        let mut command_args = args;
        self.ensure_idb_connected(id).await?;
        command_args.extend(["--udid".to_string(), id.raw.clone()]);
        self.run_logged_timeout(op, &self.config.idb_path, &command_args, timeout)
            .await
            .map(|_| ())
            .map_err(|error| match error {
                AppError::ToolMissing(_) => AppError::UnsupportedCapability(
                    "idb is required for full iOS simulator touch control; install fb-idb/idb-companion and set APPMON_IDB_PATH if needed".to_string(),
                ),
                AppError::Io(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    AppError::UnsupportedCapability(
                        "idb is required for full iOS simulator touch control; install fb-idb/idb-companion and set APPMON_IDB_PATH if needed".to_string(),
                    )
                }
                AppError::CommandFailed { stderr, .. }
                    if stderr.contains("No such file")
                        || stderr.contains("not found")
                        || stderr.contains("No target")
                        || stderr.contains("not connected") =>
                {
                    AppError::UnsupportedCapability(format!(
                        "idb could not control this iOS simulator: {stderr}"
                    ))
                }
                other => other,
            })
    }

    async fn map_ios_point(
        &self,
        id: &DeviceId,
        x: u32,
        y: u32,
        source_width: Option<u32>,
        source_height: Option<u32>,
    ) -> AppResult<(u32, u32)> {
        let Some(source_width) = source_width.filter(|value| *value > 0) else {
            return Ok((x, y));
        };
        let Some(source_height) = source_height.filter(|value| *value > 0) else {
            return Ok((x, y));
        };
        let points = self.ios_point_dimensions(id).await?;
        Ok((
            scale_coordinate(x, source_width, points.width),
            scale_coordinate(y, source_height, points.height),
        ))
    }

    async fn ios_point_dimensions(&self, id: &DeviceId) -> AppResult<PointDimensions> {
        if let Some(dimensions) = self.ios_points.lock().await.get(&id.raw).copied() {
            return Ok(dimensions);
        }

        self.ensure_idb_connected(id).await?;
        let args = vec!["describe".to_string(), "--udid".to_string(), id.raw.clone()];
        let output = self
            .run_logged_timeout(
                "ios_idb_describe",
                &self.config.idb_path,
                &args,
                Duration::from_secs(3),
            )
            .await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let dimensions = parse_idb_point_dimensions(&stdout).unwrap_or(PointDimensions {
            width: 390,
            height: 844,
        });
        self.ios_points
            .lock()
            .await
            .insert(id.raw.clone(), dimensions);
        Ok(dimensions)
    }

    async fn ensure_idb_connected(&self, id: &DeviceId) -> AppResult<()> {
        {
            let companions = self.ios_companions.lock().await;
            if companions.contains(&id.raw) {
                return Ok(());
            }
        }

        let args = vec!["connect".to_string(), id.raw.clone()];
        self.run_logged_timeout(
            "ios_idb_connect",
            &self.config.idb_path,
            &args,
            Duration::from_secs(5),
        )
        .await
        .map_err(|error| match error {
            AppError::ToolMissing(_) => AppError::UnsupportedCapability(
                "idb is required for full iOS simulator touch control; install fb-idb/idb-companion and set APPMON_IDB_PATH if needed".to_string(),
            ),
            AppError::Io(error) if error.kind() == std::io::ErrorKind::NotFound => {
                AppError::UnsupportedCapability(
                    "idb is required for full iOS simulator touch control; install fb-idb/idb-companion and set APPMON_IDB_PATH if needed".to_string(),
                )
            }
            other => other,
        })?;

        self.ios_companions.lock().await.insert(id.raw.clone());
        Ok(())
    }

    async fn run_ios_window_tap(&self, req: TapRequest) -> AppResult<()> {
        let source_width = req.source_width.unwrap_or(1).max(1);
        let source_height = req.source_height.unwrap_or(1).max(1);
        let script = format!(
            r#"tell application "Simulator" to activate
tell application "System Events"
  tell process "Simulator"
    set frontmost to true
    set {{x1, y1, x2, y2}} to bounds of window 1
    set appmonX to x1 + (((x2 - x1) * {x}) / {source_width})
    set appmonY to y1 + (((y2 - y1) * {y}) / {source_height})
    click at {{appmonX as integer, appmonY as integer}}
  end tell
end tell"#,
            x = req.x,
            y = req.y,
            source_width = source_width,
            source_height = source_height
        );
        let args = vec!["-e".to_string(), script];
        match self
            .run_logged("ios_tap_simulator_window", &self.config.osascript_path, &args)
            .await
        {
            Err(AppError::CommandFailed { stderr, .. })
                if stderr.contains("not allowed assistive access")
                    || stderr.contains("-1719")
                    || stderr.contains("-10827") =>
            {
                Err(AppError::UnsupportedCapability(
                    "iOS simulator tap needs macOS Accessibility permission for osascript/System Events".to_string(),
                ))
            }
            other => other.map(|_| ()),
        }
    }

    async fn run_logged(
        &self,
        op: &'static str,
        program: &std::path::Path,
        args: &[String],
    ) -> AppResult<CommandOutput> {
        let start = Instant::now();
        let result = self.runner.run(program, args).await;
        info!(
            op,
            program = %program.display(),
            duration_ms = start.elapsed().as_millis(),
            success = result.is_ok(),
            "device command completed"
        );
        result
    }

    async fn run_logged_timeout(
        &self,
        op: &'static str,
        program: &std::path::Path,
        args: &[String],
        timeout: Duration,
    ) -> AppResult<CommandOutput> {
        let start = Instant::now();
        let result = tokio::time::timeout(timeout, self.runner.run(program, args))
            .await
            .map_err(|_| AppError::CommandFailed {
                program: program.display().to_string(),
                args: args.to_vec(),
                status: None,
                stderr: format!("command timed out after {}ms", timeout.as_millis()),
            })
            .and_then(|result| result);
        info!(
            op,
            program = %program.display(),
            duration_ms = start.elapsed().as_millis(),
            success = result.is_ok(),
            "device command completed"
        );
        result
    }
}

pub trait DeviceController {}
impl<R: ProcessRunner> DeviceController for DeviceManager<R> {}

struct RecordingJob {
    output_path: PathBuf,
    child: tokio::process::Child,
}

struct AndroidInputSession {
    stdin: ChildStdin,
    child: tokio::process::Child,
}

#[derive(Clone, Copy, Debug)]
struct PointDimensions {
    width: u32,
    height: u32,
}

impl AndroidInputSession {
    fn spawn(program: &std::path::Path, serial: &str) -> AppResult<Self> {
        if !program.exists() {
            return Err(AppError::ToolMissing(program.display().to_string()));
        }
        let mut child = Command::new(program)
            .args(["-s", serial, "shell"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::InvalidInput("failed to open adb shell stdin".to_string()))?;
        Ok(Self { stdin, child })
    }

    async fn write_command(&mut self, command: &str) -> AppResult<()> {
        if let Some(status) = self.child.try_wait()? {
            return Err(AppError::CommandFailed {
                program: "adb".to_string(),
                args: vec!["shell".to_string()],
                status: status.code(),
                stderr: "persistent adb shell exited".to_string(),
            });
        }
        self.stdin.write_all(command.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }
}

fn spawn_recording(program: &std::path::Path, args: &[String]) -> AppResult<tokio::process::Child> {
    if !program.exists() {
        return Err(AppError::ToolMissing(program.display().to_string()));
    }
    Ok(Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?)
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '%' | ':' | '/' | ','))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn scale_coordinate(value: u32, source: u32, target: u32) -> u32 {
    ((u64::from(value) * u64::from(target)) / u64::from(source))
        .min(u64::from(target.saturating_sub(1)))
        .try_into()
        .unwrap_or(target.saturating_sub(1))
}

fn android_motion_action(action: MotionAction) -> &'static str {
    match action {
        MotionAction::Down => "DOWN",
        MotionAction::Move => "MOVE",
        MotionAction::Up => "UP",
    }
}

fn android_grpc_port_from_endpoint(endpoint: &str) -> Option<u16> {
    let without_scheme = endpoint
        .strip_prefix("http://")
        .or_else(|| endpoint.strip_prefix("https://"))
        .unwrap_or(endpoint);
    let authority = without_scheme.split('/').next().unwrap_or(without_scheme);
    authority.rsplit_once(':')?.1.parse().ok()
}

fn parse_idb_point_dimensions(input: &str) -> Option<PointDimensions> {
    let width = parse_idb_dimension(input, "width_points")?;
    let height = parse_idb_dimension(input, "height_points")?;
    Some(PointDimensions { width, height })
}

fn parse_idb_dimension(input: &str, key: &str) -> Option<u32> {
    let start = input.find(key)? + key.len();
    let rest = input[start..].trim_start();
    let rest = rest.strip_prefix('=').unwrap_or(rest).trim_start();
    let digits = rest
        .chars()
        .take_while(|value| value.is_ascii_digit())
        .collect::<String>();
    digits.parse().ok()
}

fn ios_button_for_key(key: &str) -> Option<&'static str> {
    match key.to_ascii_uppercase().as_str() {
        "HOME" => Some("HOME"),
        "LOCK" | "SLEEP" => Some("LOCK"),
        "SIRI" => Some("SIRI"),
        "SIDE_BUTTON" | "SIDE-BUTTON" => Some("SIDE_BUTTON"),
        "APPLE_PAY" | "APPLE-PAY" => Some("APPLE_PAY"),
        "BACK" => Some("HOME"),
        _ => None,
    }
}

fn is_ios_app_switch_key(key: &str) -> bool {
    matches!(key.to_ascii_uppercase().as_str(), "APP_SWITCH" | "RECENTS")
}

pub fn parse_adb_devices(input: &str) -> Vec<Device> {
    input
        .lines()
        .skip(1)
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let mut parts = line.split_whitespace();
            let serial = parts.next()?;
            let state = parts.next()?;
            let details = parts.map(ToString::to_string).collect::<Vec<_>>();
            let name = details
                .iter()
                .find_map(|part| part.strip_prefix("model:"))
                .unwrap_or(serial)
                .replace('_', " ");
            Some(Device {
                id: DeviceId {
                    kind: DeviceKind::Android,
                    raw: serial.to_string(),
                }
                .web_id(),
                kind: DeviceKind::Android,
                name,
                state: state.to_string(),
                details,
            })
        })
        .collect()
}

pub fn parse_android_avds(input: &str) -> Vec<Device> {
    input
        .lines()
        .filter_map(|line| {
            let avd = line.trim();
            if avd.is_empty() {
                return None;
            }
            Some(Device {
                id: DeviceId {
                    kind: DeviceKind::Android,
                    raw: avd.to_string(),
                }
                .web_id(),
                kind: DeviceKind::Android,
                name: avd.replace('_', " "),
                state: "Shutdown".to_string(),
                details: vec!["avd".to_string()],
            })
        })
        .collect()
}

pub fn parse_simctl_devices(input: &[u8]) -> AppResult<Vec<Device>> {
    #[derive(Deserialize)]
    struct SimctlRoot {
        devices: std::collections::BTreeMap<String, Vec<SimctlDevice>>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SimctlDevice {
        name: String,
        udid: String,
        state: String,
        is_available: Option<bool>,
        availability_error: Option<String>,
    }

    let root: SimctlRoot = serde_json::from_slice(input)?;
    let devices = root
        .devices
        .into_iter()
        .flat_map(|(runtime, devices)| {
            devices.into_iter().filter_map(move |device| {
                if device.is_available == Some(false) {
                    return None;
                }
                let mut details = vec![runtime.clone()];
                if let Some(error) = device.availability_error {
                    details.push(error);
                }
                Some(Device {
                    id: DeviceId {
                        kind: DeviceKind::Ios,
                        raw: device.udid,
                    }
                    .web_id(),
                    kind: DeviceKind::Ios,
                    name: device.name,
                    state: device.state,
                    details,
                })
            })
        })
        .collect();
    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_adb_devices() {
        let input = "List of devices attached\nemulator-5554 device product:sdk model:Pixel_8 device:emu\nabc offline\n";
        let devices = parse_adb_devices(input);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].id, "android:emulator-5554");
        assert_eq!(devices[0].name, "Pixel 8");
        assert_eq!(devices[1].id, "android:abc");
        assert_eq!(devices[1].state, "offline");
    }

    #[test]
    fn parses_android_avds() {
        let devices = parse_android_avds("Pixel_8_API_35\nTablet API 35\n\n");
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].id, "android:Pixel_8_API_35");
        assert_eq!(devices[0].name, "Pixel 8 API 35");
        assert_eq!(devices[0].state, "Shutdown");
        assert_eq!(devices[0].details, ["avd"]);
    }

    #[test]
    fn parses_simctl_available_devices() {
        let input = br#"{
          "devices": {
            "com.apple.CoreSimulator.SimRuntime.iOS-17-5": [
              {"name":"iPhone 15","udid":"A-B-C","state":"Booted","isAvailable":true},
              {"name":"iPhone 14","udid":"D-E-F","state":"Shutdown","isAvailable":true},
              {"name":"iPhone 13","udid":"G-H-I","state":"Shutdown","isAvailable":false}
            ]
          }
        }"#;
        let devices = parse_simctl_devices(input).unwrap();
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].id, "ios:A-B-C");
        assert_eq!(devices[1].id, "ios:D-E-F");
        assert_eq!(devices[1].state, "Shutdown");
    }

    #[test]
    fn validates_device_ids() {
        assert!(DeviceId::parse("android:emulator-5554").is_ok());
        assert!(DeviceId::parse("ios:abc").is_ok());
        assert!(DeviceId::parse("abc").is_err());
    }

    #[test]
    fn quotes_shell_arguments_for_persistent_input() {
        assert_eq!(shell_quote("BACK"), "BACK");
        assert_eq!(shell_quote("hello%s"), "hello%s");
        assert_eq!(shell_quote("can't"), "'can'\\''t'");
    }

    #[test]
    fn parses_idb_point_dimensions() {
        let output = "name=iPhone width_points=430 height_points=932 other=value";
        let dimensions = parse_idb_point_dimensions(output).unwrap();
        assert_eq!(dimensions.width, 430);
        assert_eq!(dimensions.height, 932);
    }

    #[test]
    fn scales_coordinates_to_idb_points() {
        assert_eq!(scale_coordinate(660, 1320, 430), 215);
        assert_eq!(scale_coordinate(2868, 2868, 932), 931);
    }

    #[test]
    fn extracts_android_grpc_port_from_endpoint() {
        assert_eq!(
            android_grpc_port_from_endpoint("http://127.0.0.1:8554"),
            Some(8554)
        );
        assert_eq!(
            android_grpc_port_from_endpoint("https://localhost:9554/foo"),
            Some(9554)
        );
        assert_eq!(android_grpc_port_from_endpoint("localhost"), None);
    }
}
