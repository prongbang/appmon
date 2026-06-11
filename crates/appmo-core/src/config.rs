use std::{env, net::SocketAddr, path::PathBuf};

use crate::{AppError, AppResult};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bind: SocketAddr,
    pub adb_path: PathBuf,
    pub xcrun_path: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        let bind = env::var("APPMO_BIND")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
            .parse()
            .map_err(|_| AppError::InvalidInput("APPMO_BIND must be host:port".to_string()))?;

        Ok(Self {
            bind,
            adb_path: env::var("ANDROID_ADB_PATH")
                .unwrap_or_else(|_| {
                    "/Users/inteniquetic/Library/Android/sdk/platform-tools/adb".to_string()
                })
                .into(),
            xcrun_path: env::var("IOS_XCRUN_PATH")
                .unwrap_or_else(|_| "/usr/bin/xcrun".to_string())
                .into(),
        })
    }
}
