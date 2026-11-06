/*
  build.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

fn main() {
    println!("cargo:rerun-if-env-changed=BUILD_DATE");
    println!("cargo:rerun-if-changed=assets/aiqiezi.ico");
    println!("cargo:rerun-if-changed=assets/aiqiezi.png");

    let build_date = std::env::var("BUILD_DATE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "dev".to_string());

    println!("cargo:rustc-env=AIQIEZI_BUILD_DATE={build_date}");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/aiqiezi.ico");
        resource
            .compile()
            .expect("embed assets/aiqiezi.ico as the Windows executable icon");
    }
}
