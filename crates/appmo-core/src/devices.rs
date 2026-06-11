use std::{collections::HashMap, path::PathBuf, process::Stdio, sync::Arc, time::Instant};

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use tokio::{process::Command, sync::Mutex};
use tracing::info;
use uuid::Uuid;

use crate::{
    validation, AppConfig, AppError, AppResult, CommandOutput, ProcessRunner, TokioProcessRunner,
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

#[derive(Clone, Debug, Deserialize)]
pub struct TapRequest {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SwipeRequest {
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
    pub duration_ms: Option<u32>,
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
        }
    }

    pub async fn list_devices(&self) -> AppResult<Vec<Device>> {
        let mut devices = Vec::new();
        devices.extend(self.list_android_devices().await.unwrap_or_default());
        devices.extend(self.list_ios_devices().await.unwrap_or_default());
        Ok(devices)
    }

    pub async fn screenshot(&self, id: &DeviceId) -> AppResult<Vec<u8>> {
        match id.kind {
            DeviceKind::Android => {
                let args = vec![
                    "-s".to_string(),
                    id.raw.clone(),
                    "exec-out".to_string(),
                    "screencap".to_string(),
                    "-p".to_string(),
                ];
                Ok(self
                    .run_logged("android_screenshot", &self.config.adb_path, &args)
                    .await?
                    .stdout)
            }
            DeviceKind::Ios => {
                let file = NamedTempFile::new()?;
                let path = file.path().to_path_buf();
                let args = vec![
                    "simctl".to_string(),
                    "io".to_string(),
                    id.raw.clone(),
                    "screenshot".to_string(),
                    path.display().to_string(),
                ];
                self.run_logged("ios_screenshot", &self.config.xcrun_path, &args)
                    .await?;
                Ok(tokio::fs::read(path).await?)
            }
        }
    }

    pub async fn tap(&self, id: &DeviceId, req: TapRequest) -> AppResult<()> {
        validation::coordinate(req.x, "x")?;
        validation::coordinate(req.y, "y")?;
        match id.kind {
            DeviceKind::Android => {
                let args = vec![
                    "-s".to_string(),
                    id.raw.clone(),
                    "shell".to_string(),
                    "input".to_string(),
                    "tap".to_string(),
                    req.x.to_string(),
                    req.y.to_string(),
                ];
                self.run_logged("android_tap", &self.config.adb_path, &args)
                    .await?;
            }
            DeviceKind::Ios => {
                let args = vec![
                    "simctl".to_string(),
                    "io".to_string(),
                    id.raw.clone(),
                    "tap".to_string(),
                    req.x.to_string(),
                    req.y.to_string(),
                ];
                self.run_ios_input("ios_tap", args).await?;
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
                let args = vec![
                    "-s".to_string(),
                    id.raw.clone(),
                    "shell".to_string(),
                    "input".to_string(),
                    "swipe".to_string(),
                    req.x1.to_string(),
                    req.y1.to_string(),
                    req.x2.to_string(),
                    req.y2.to_string(),
                    duration.to_string(),
                ];
                self.run_logged("android_swipe", &self.config.adb_path, &args)
                    .await?;
            }
            DeviceKind::Ios => {
                let args = vec![
                    "simctl".to_string(),
                    "io".to_string(),
                    id.raw.clone(),
                    "swipe".to_string(),
                    req.x1.to_string(),
                    req.y1.to_string(),
                    req.x2.to_string(),
                    req.y2.to_string(),
                    duration.to_string(),
                ];
                self.run_ios_input("ios_swipe", args).await?;
            }
        }
        Ok(())
    }

    pub async fn text(&self, id: &DeviceId, req: TextRequest) -> AppResult<()> {
        validation::text_input(&req.text)?;
        match id.kind {
            DeviceKind::Android => {
                let escaped = req.text.replace(' ', "%s");
                let args = vec![
                    "-s".to_string(),
                    id.raw.clone(),
                    "shell".to_string(),
                    "input".to_string(),
                    "text".to_string(),
                    escaped,
                ];
                self.run_logged("android_text", &self.config.adb_path, &args)
                    .await?;
            }
            DeviceKind::Ios => {
                return Err(AppError::UnsupportedCapability(
                    "simctl does not provide reliable text input in this v1".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub async fn key(&self, id: &DeviceId, req: KeyRequest) -> AppResult<()> {
        validation::non_empty(&req.key, "key")?;
        match id.kind {
            DeviceKind::Android => {
                let args = vec![
                    "-s".to_string(),
                    id.raw.clone(),
                    "shell".to_string(),
                    "input".to_string(),
                    "keyevent".to_string(),
                    req.key,
                ];
                self.run_logged("android_key", &self.config.adb_path, &args)
                    .await?;
            }
            DeviceKind::Ios => {
                let args = vec![
                    "simctl".to_string(),
                    "io".to_string(),
                    id.raw.clone(),
                    "key".to_string(),
                    req.key,
                ];
                self.run_ios_input("ios_key", args).await?;
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
                let remote_path = PathBuf::from(format!("/sdcard/appmo-{}.mp4", Uuid::new_v4()));
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
                    std::env::temp_dir().join(format!("appmo-ios-{}.mp4", Uuid::new_v4()))
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
        Ok(parse_adb_devices(&String::from_utf8_lossy(&output.stdout)))
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

    async fn run_ios_input(&self, op: &'static str, args: Vec<String>) -> AppResult<CommandOutput> {
        match self.run_logged(op, &self.config.xcrun_path, &args).await {
            Err(AppError::CommandFailed { stderr, .. })
                if stderr.contains("Invalid device command")
                    || stderr.contains("Usage:")
                    || stderr.contains("unrecognized") =>
            {
                Err(AppError::UnsupportedCapability(
                    "simctl input is not supported by the installed Xcode command line tools"
                        .to_string(),
                ))
            }
            other => other,
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
}

pub trait DeviceController {}
impl<R: ProcessRunner> DeviceController for DeviceManager<R> {}

struct RecordingJob {
    output_path: PathBuf,
    child: tokio::process::Child,
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
            if state != "device" {
                return None;
            }
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
                if device.state != "Booted" || device.is_available == Some(false) {
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
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "android:emulator-5554");
        assert_eq!(devices[0].name, "Pixel 8");
    }

    #[test]
    fn parses_simctl_booted_devices() {
        let input = br#"{
          "devices": {
            "com.apple.CoreSimulator.SimRuntime.iOS-17-5": [
              {"name":"iPhone 15","udid":"A-B-C","state":"Booted","isAvailable":true},
              {"name":"iPhone 14","udid":"D-E-F","state":"Shutdown","isAvailable":true}
            ]
          }
        }"#;
        let devices = parse_simctl_devices(input).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "ios:A-B-C");
    }

    #[test]
    fn validates_device_ids() {
        assert!(DeviceId::parse("android:emulator-5554").is_ok());
        assert!(DeviceId::parse("ios:abc").is_ok());
        assert!(DeviceId::parse("abc").is_err());
    }
}
