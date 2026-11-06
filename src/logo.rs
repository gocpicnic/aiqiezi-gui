/*
  src/logo.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use eframe::egui;

pub struct Logo {
    pub icon: egui::IconData,
    pub image: egui::ColorImage,
}

pub fn load() -> Logo {
    let bytes = include_bytes!("../assets/aiqiezi.png");
    let decoded = image::load_from_memory(bytes)
        .expect("decode embedded aiqiezi logo")
        .to_rgba8();
    let width = decoded.width();
    let height = decoded.height();
    let rgba = decoded.as_raw();

    Logo {
        icon: egui::IconData {
            rgba: rgba.clone(),
            width,
            height,
        },
        image: egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], rgba),
    }
}

#[cfg(test)]
mod tests {
    use super::load;

    #[test]
    fn embedded_logo_decodes() {
        let logo = load();
        assert_eq!(logo.icon.width, 340);
        assert_eq!(logo.icon.height, 340);
        assert_eq!(logo.image.size, [340, 340]);
        assert_eq!(logo.icon.rgba.len(), 340 * 340 * 4);
    }
}
