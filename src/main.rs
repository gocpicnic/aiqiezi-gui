/*
  src/main.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

mod app;
mod daemon_process;
mod logging;
mod logo;
mod theme;
mod version;
mod worker;

use eframe::egui;

fn main() -> eframe::Result {
    logging::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(1280.0, 800.0))
            .with_min_inner_size(egui::vec2(980.0, 640.0))
            .with_icon(logo::icon()),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    #[cfg(target_os = "linux")]
    let options = {
        let mut options = options;
        if std::env::var_os("DISPLAY").is_some() && std::env::var_os("AIQIEZI_WAYLAND").is_none() {
            tracing::info!("using X11/XWayland event loop");
            options.event_loop_builder = Some(Box::new(|builder| {
                use winit::platform::x11::EventLoopBuilderExtX11;
                builder.with_x11();
            }));
        } else {
            tracing::info!("using native Wayland event loop");
        }
        options
    };

    eframe::run_native(
        "aiqiezi-gui",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
