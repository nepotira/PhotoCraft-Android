//! PhotoCraft on Android using eframe's NativeActivity runner.

use photocraft_ui_egui::theme::ThemeKind;
use photocraft_ui_egui::{PhotocraftApp, Services};
use winit::platform::android::activity::AndroidApp;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    log::info!("PhotoCraft Android starting up");

    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };

    let result = eframe::run_native(
        "PhotoCraft",
        options,
        Box::new(|cc| {
            PhotocraftApp::setup_context(&cc.egui_ctx, ThemeKind::Pro);
            let session = photocraft_engine::Session::default();
            let services = Services::default();
            let mut photocraft = PhotocraftApp::new(session, services);
            photocraft.set_theme(&cc.egui_ctx, ThemeKind::Pro);
            if let Some(rs) = cc.wgpu_render_state.clone() {
                photocraft.set_wgpu(rs);
            }
            Ok(Box::new(photocraft))
        }),
    );

    if let Err(e) = result {
        log::error!("PhotoCraft exited with error: {e}");
    }
}
