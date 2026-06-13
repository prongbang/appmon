use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Context, Result};
use tracing::{info, warn};

pub fn ensure_runtime_dependencies() {
    if !auto_install_enabled() {
        info!("runtime dependency auto-install disabled");
        return;
    }

    if !program_available("/usr/bin/xcrun") && !program_available("xcrun") {
        warn!(
            "xcrun is missing; install Xcode or Xcode Command Line Tools for iOS simulator support"
        );
    }

    ensure_android_tools();
    ensure_ios_tools();
    ensure_media_tools();
}

fn auto_install_enabled() -> bool {
    env::var("APPMON_AUTO_INSTALL_DEPS")
        .or_else(|_| env::var("APPMO_AUTO_INSTALL_DEPS"))
        .map(|value| {
            !matches!(
                value.to_ascii_lowercase().as_str(),
                "0" | "false" | "off" | "no"
            )
        })
        .unwrap_or(true)
}

fn ensure_android_tools() {
    let sdk_adb = env::var("ANDROID_HOME")
        .or_else(|_| env::var("ANDROID_SDK_ROOT"))
        .ok()
        .map(PathBuf::from)
        .map(|path| path.join("platform-tools").join("adb"));
    let home_adb = env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .map(|path| path.join("Library/Android/sdk/platform-tools/adb"));

    if sdk_adb.as_deref().is_some_and(path_exists)
        || home_adb.as_deref().is_some_and(path_exists)
        || program_available("adb")
    {
        return;
    }

    if let Err(error) = run_brew(["install", "android-platform-tools"]) {
        warn!(%error, "could not auto-install Android platform tools; install Android SDK platform-tools or set ANDROID_ADB_PATH");
    }
}

fn ensure_ios_tools() {
    if !program_available("idb") && !home_bin("idb").exists() {
        if let Err(error) = install_idb() {
            warn!(%error, "could not auto-install idb; iOS full touch control will need manual setup");
        }
    }
}

fn ensure_media_tools() {
    if program_available("ffmpeg") {
        return;
    }

    if let Err(error) = run_brew(["install", "ffmpeg"]) {
        warn!(%error, "could not auto-install ffmpeg; WebRTC video preview will fall back to data-channel streaming");
    }
}

fn install_idb() -> Result<()> {
    ensure_brew()?;
    run_brew(["tap", "facebook/fb"]).context("brew tap facebook/fb failed")?;
    run_brew(["trust", "facebook/fb"]).context("brew trust facebook/fb failed")?;
    run_brew(["install", "idb-companion"]).context("brew install idb-companion failed")?;

    if !program_available("pipx") {
        run_brew(["install", "pipx"]).context("brew install pipx failed")?;
    }

    let python = preferred_python();
    let mut args = vec!["install".to_string()];
    if let Some(python) = python {
        args.extend(["--python".to_string(), python.display().to_string()]);
    }
    args.push("fb-idb".to_string());
    run_command("pipx", args).context("pipx install fb-idb failed")?;
    Ok(())
}

fn ensure_brew() -> Result<()> {
    if program_available("brew") {
        Ok(())
    } else {
        Err(anyhow!(
            "Homebrew is required for automatic dependency installation"
        ))
    }
}

fn run_brew<const N: usize>(args: [&str; N]) -> Result<()> {
    run_command("brew", args.iter().copied())
}

fn run_command<I, S>(program: &str, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    info!(program, ?args, "running dependency bootstrap command");
    let output = Command::new(program)
        .args(&args)
        .output()
        .with_context(|| format!("failed to start {program}"))?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(anyhow!(
        "{program} exited with status {:?}: {stderr}",
        output.status.code()
    ))
}

fn preferred_python() -> Option<PathBuf> {
    [
        home_bin("python3.11"),
        PathBuf::from("/opt/homebrew/bin/python3.13"),
        PathBuf::from("/opt/homebrew/bin/python3.12"),
        PathBuf::from("/opt/homebrew/bin/python3.11"),
    ]
    .into_iter()
    .find(|path| path.exists())
    .or_else(|| find_in_path("python3.13"))
    .or_else(|| find_in_path("python3.12"))
    .or_else(|| find_in_path("python3.11"))
    .or_else(|| find_in_path("python3"))
}

fn home_bin(name: &str) -> PathBuf {
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".local/bin")
        .join(name)
}

fn program_available(program: &str) -> bool {
    let path = Path::new(program);
    if path.components().count() > 1 {
        return path.exists();
    }
    find_in_path(program).is_some()
}

fn find_in_path(program: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|path| path.join(program))
            .find(|path| path.exists())
    })
}

fn path_exists(path: &Path) -> bool {
    path.exists()
}
