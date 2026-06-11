use std::{path::Path, process::Stdio};

use async_trait::async_trait;
use tokio::process::Command;

use crate::{AppError, AppResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandOutput {
    pub status: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: String,
}

#[async_trait]
pub trait ProcessRunner: Send + Sync + 'static {
    async fn run(&self, program: &Path, args: &[String]) -> AppResult<CommandOutput>;
}

#[derive(Clone, Default)]
pub struct TokioProcessRunner;

#[async_trait]
impl ProcessRunner for TokioProcessRunner {
    async fn run(&self, program: &Path, args: &[String]) -> AppResult<CommandOutput> {
        if !program.exists() {
            return Err(AppError::ToolMissing(program.display().to_string()));
        }

        let output = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let status = output.status.code();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if !output.status.success() {
            return Err(AppError::CommandFailed {
                program: program.display().to_string(),
                args: args.to_vec(),
                status,
                stderr,
            });
        }

        Ok(CommandOutput {
            status,
            stdout: output.stdout,
            stderr,
        })
    }
}
