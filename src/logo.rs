/*
  src/logo.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use eframe::egui;

pub fn icon() -> egui::IconData {
    let bytes = include_bytes!("../assets/aiqiezi.png");
    let decoded = image::load_from_memory(bytes)
        .expect("decode embedded aiqiezi icon")
        .to_rgba8();
    let width = decoded.width();
    let height = decoded.height();
    let rgba = decoded.as_raw();

    egui::IconData {
        rgba: rgba.clone(),
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::icon;

    #[test]
    fn embedded_icon_decodes() {
        let icon = icon();
        assert_eq!(icon.width, 340);
        assert_eq!(icon.height, 340);
        assert_eq!(icon.rgba.len(), 340 * 340 * 4);
    }
}
