//! PhotoCraft on Android using eframe's NativeActivity runner.
//!
//! Port maintained by Nepo (@nepotira).
//! Version: 0.0.1-android (based on PhotoCraft upstream v0.8.2).

mod onboarding;

use onboarding::{OnboardingData, render_wizard};
use photocraft_ui_egui::theme::ThemeKind;
use photocraft_ui_egui::{PhotocraftApp, Services};
use winit::platform::android::activity::AndroidApp;

enum AndroidState {
    Onboarding(OnboardingData),
    Running(Box<PhotocraftApp>),
}

struct AndroidShell {
    state: AndroidState,
    ui_scale: f32,
    session: Option<photocraft_engine::Session>,
    services: Option<Services>,
    wgpu_state: Option<eframe::egui_wgpu::RenderState>,
}

impl AndroidShell {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        PhotocraftApp::setup_context(&cc.egui_ctx, ThemeKind::Pro);
        let session = photocraft_engine::Session::default();
        let services = Services::default();

        Self {
            state: AndroidState::Onboarding(OnboardingData::default()),
            ui_scale: 0.85,
            session: Some(session),
            services: Some(services),
            wgpu_state: cc.wgpu_render_state.clone(),
        }
    }
}

impl eframe::App for AndroidShell {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Enforce calibrated touch/DPI scale so elements fit the viewport
        ctx.set_pixels_per_point(self.ui_scale);

        match &mut self.state {
            AndroidState::Onboarding(data) => {
                render_wizard(ctx, data);
                self.ui_scale = data.ui_scale;

                if data.finished {
                    let session = self.session.take().unwrap_or_default();
                    let services = self.services.take().unwrap_or_default();
                    let mut app = PhotocraftApp::new(session, services);
                    app.set_theme(ctx, ThemeKind::Pro);
                    if let Some(rs) = self.wgpu_state.clone() {
                        app.set_wgpu(rs);
                    }
                    self.state = AndroidState::Running(Box::new(app));
                }
            }
            AndroidState::Running(app) => {
                // Persistent top mobile bar with touch zoom controls and DeX helpers
                egui::TopBottomPanel::top("android_mobile_bar").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🎨 PhotoCraft Android").small().strong());
                        ui.separator();
                        ui.label(egui::RichText::new("Zoom da UI:").small());
                        if ui.button("➖").clicked() && self.ui_scale > 0.5 {
                            self.ui_scale -= 0.05;
                        }
                        ui.label(egui::RichText::new(format!("{:.0}%", self.ui_scale * 100.0)).small());
                        if ui.button("➕").clicked() && self.ui_scale < 1.6 {
                            self.ui_scale += 0.05;
                        }
                        ui.separator();
                        if ui.button(egui::RichText::new("⚙️ Configurar").small()).clicked() {
                            let mut data = OnboardingData::default();
                            data.ui_scale = self.ui_scale;
                            data.storage_granted = true;
                            self.state = AndroidState::Onboarding(data);
                        }
                    });
                });

                // Run main Photoshop/PhotoCraft interface
                app.update(ctx, frame);
            }
        }
    }
}

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
        Box::new(|cc| Ok(Box::new(AndroidShell::new(cc)))),
    );

    if let Err(e) = result {
        log::error!("PhotoCraft exited with error: {e}");
    }
}
