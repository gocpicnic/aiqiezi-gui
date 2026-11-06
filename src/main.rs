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
mod version;
mod worker;

use eframe::egui;

fn main() -> eframe::Result {
    logging::init();

    let logo = logo::load();
    let splash_image = logo.image;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(980.0, 720.0))
            .with_min_inner_size(egui::vec2(760.0, 540.0))
            .with_icon(logo.icon),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "aiqiezi",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, splash_image)))),
    )
}
