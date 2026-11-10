/*
  src/daemon_process.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use aiqiezi_client::config::create_private_dir;
use aiqiezi_client::ClientConfig;
use tracing::info;

pub struct SpawnedDaemon {
    pub child: Child,
    pub log_path: PathBuf,
}

pub fn spawn_daemon(
    config: &ClientConfig,
    endpoint_id: Option<&str>,
) -> Result<SpawnedDaemon, String> {
    let cli = find_cli().ok_or_else(|| {
        "could not find the aiqiezi client daemon; run `make build`, or set \
         `AIQIEZI_CLI_BIN` to the daemon binary"
            .to_string()
    })?;

    create_private_dir(&config.data_dir).map_err(|error| {
        format!(
            "create data directory {}: {error:#}",
            config.data_dir.display()
        )
    })?;

    let log_path = client_log_path(config);
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|error| format!("open client log {}: {error}", log_path.display()))?;
    let stderr = log
        .try_clone()
        .map_err(|error| format!("clone client log handle: {error}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&log_path, fs::Permissions::from_mode(0o600))
            .map_err(|error| format!("set permissions on {}: {error}", log_path.display()))?;
    }

    info!(
        cli = %cli.display(),
        endpoint_supplied = endpoint_id.is_some(),
        data_dir = %config.data_dir.display(),
        runtime_dir = %config.runtime_dir.display(),
        log = %log_path.display(),
        "spawning local client daemon"
    );

    let mut command = Command::new(&cli);
    command.arg("start");
    if let Some(endpoint_id) = endpoint_id {
        command.arg(endpoint_id);
    }
    command
        .env("AIQIEZI_DATA_DIR", &config.data_dir)
        .env("AIQIEZI_RUNTIME_DIR", &config.runtime_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(stderr));

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let child = command
        .spawn()
        .map_err(|error| format!("start {}: {error}", cli.display()))?;
    info!(pid = child.id(), "local client daemon started");

    Ok(SpawnedDaemon { child, log_path })
}

pub fn client_log_path(config: &ClientConfig) -> PathBuf {
    config.data_dir.join("client.log")
}

pub fn find_cli() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("AIQIEZI_CLI_BIN") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let candidate = parent.join(cli_filename());
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|directory| directory.join(cli_filename()))
            .find(|candidate| candidate.is_file())
    })
}

fn cli_filename() -> &'static str {
    if cfg!(windows) {
        "aiqiezi.exe"
    } else {
        "aiqiezi"
    }
}
