/*
  src/logging.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::io::IsTerminal;

use tracing_subscriber::EnvFilter;

pub fn init() {
    let filter = std::env::var("AIQIEZI_LOG_LEVEL")
        .unwrap_or_else(|_| "aiqiezi_gui=info,aiqiezi_client=info,iroh=warn".to_string());
    let filter = match EnvFilter::try_new(filter) {
        Ok(filter) => filter,
        Err(error) => {
            eprintln!("invalid log filter: {error}");
            return;
        }
    };

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .try_init();
}
