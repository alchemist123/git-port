use std::process::{Command, Output, Stdio};

use anyhow::Context;

pub fn git(args: &[&str]) -> anyhow::Result<String> {
    let output = git_output(args)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git {:?} failed ({}): {}", args, output.status, stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim_end().to_string())
}

pub fn git_output(args: &[&str]) -> anyhow::Result<Output> {
    Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("failed to spawn git {:?}", args))
}

pub fn git_live(args: &[&str]) -> anyhow::Result<bool> {
    let status = Command::new("git")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to spawn git {:?}", args))?;

    Ok(status.success())
}
