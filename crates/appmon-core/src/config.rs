use std::{env, net::SocketAddr, path::PathBuf};

use crate::{AppError, AppResult};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bind: SocketAddr,
    pub udp_bind: Option<SocketAddr>,
    pub adb_path: PathBuf,
    pub emulator_path: PathBuf,
    pub android_grpc_endpoint: Option<String>,
    pub xcrun_path: PathBuf,
    pub osascript_path: PathBuf,
    pub idb_path: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        let bind: SocketAddr = appmon_env("APPMON_BIND", "APPMO_BIND")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
            .parse()
            .map_err(|_| AppError::InvalidInput("APPMON_BIND must be host:port".to_string()))?;
        let udp_bind = match appmon_env("APPMON_UDP_BIND", "APPMO_UDP_BIND") {
            Ok(value)
                if value.eq_ignore_ascii_case("off") || value.eq_ignore_ascii_case("false") =>
            {
                None
            }
            Ok(value) => Some(value.parse().map_err(|_| {
                AppError::InvalidInput(
                    "APPMON_UDP_BIND must be host:port, off, or false".to_string(),
                )
            })?),
            Err(_) => Some(SocketAddr::new(bind.ip(), bind.port().saturating_add(1))),
        };

        Ok(Self {
            bind,
            udp_bind,
            adb_path: env::var("ANDROID_ADB_PATH")
                .unwrap_or_else(|_| default_adb_path().display().to_string())
                .into(),
            emulator_path: env::var("ANDROID_EMULATOR_PATH")
                .unwrap_or_else(|_| default_emulator_path().display().to_string())
                .into(),
            android_grpc_endpoint: appmon_env(
                "APPMON_ANDROID_GRPC_ENDPOINT",
                "APPMO_ANDROID_GRPC_ENDPOINT",
            )
            .ok()
            .filter(|value| !value.eq_ignore_ascii_case("off") && !value.is_empty()),
            xcrun_path: env::var("IOS_XCRUN_PATH")
                .unwrap_or_else(|_| "/usr/bin/xcrun".to_string())
                .into(),
            osascript_path: appmon_env("APPMON_OSASCRIPT_PATH", "APPMO_OSASCRIPT_PATH")
                .unwrap_or_else(|_| "/usr/bin/osascript".to_string())
                .into(),
            idb_path: appmon_env("APPMON_IDB_PATH", "APPMO_IDB_PATH")
                .unwrap_or_else(|_| default_idb_path().display().to_string())
                .into(),
        })
    }
}

fn appmon_env(primary: &str, legacy: &str) -> Result<String, env::VarError> {
    env::var(primary).or_else(|_| env::var(legacy))
}

fn default_adb_path() -> PathBuf {
    let android_home = env::var("ANDROID_HOME")
        .or_else(|_| env::var("ANDROID_SDK_ROOT"))
        .ok()
        .map(PathBuf::from);
    if let Some(path) = android_home
        .map(|path| path.join("platform-tools").join("adb"))
        .filter(|path| path.exists())
    {
        return path;
    }

    let home_sdk = env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .map(|home| home.join("Library/Android/sdk/platform-tools/adb"));
    if let Some(path) = home_sdk.filter(|path| path.exists()) {
        return path;
    }

    "adb".into()
}

fn default_emulator_path() -> PathBuf {
    let android_home = env::var("ANDROID_HOME")
        .or_else(|_| env::var("ANDROID_SDK_ROOT"))
        .ok()
        .map(PathBuf::from);
    if let Some(path) = android_home
        .map(|path| path.join("emulator").join("emulator"))
        .filter(|path| path.exists())
    {
        return path;
    }

    let home_sdk = env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .map(|home| home.join("Library/Android/sdk/emulator/emulator"));
    if let Some(path) = home_sdk.filter(|path| path.exists()) {
        return path;
    }

    "emulator".into()
}

fn default_idb_path() -> PathBuf {
    let user_idb = env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .map(|home| home.join(".local/bin/idb"));
    if let Some(path) = user_idb.filter(|path| path.exists()) {
        return path;
    }

    "idb".into()
}
