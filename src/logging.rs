/*
  src/logging.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::io::IsTerminal;

use tracing_subscriber::EnvFilter;

const DEFAULT_FILTER: &str =
    "aiqiezi_gui=info,aiqiezi_client=info,eframe=info,winit=warn,iroh=warn";

pub fn init() {
    let filter_spec = non_empty_env("AIQIEZI_LOG_LEVEL")
        .or_else(|| non_empty_env("RUST_LOG"))
        .unwrap_or_else(|| DEFAULT_FILTER.to_string());
    let filter = match EnvFilter::try_new(filter_spec) {
        Ok(filter) => filter,
        Err(error) => {
            eprintln!("invalid log filter: {error}");
            return;
        }
    };

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_names(true)
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .try_init();
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
