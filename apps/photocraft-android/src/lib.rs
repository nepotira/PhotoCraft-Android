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
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(self.ui_scale);
        if let AndroidState::Running(app) = &mut self.state {
            app.logic(ctx, frame);
        }
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if let AndroidState::Running(app) = &mut self.state {
            app.raw_input_hook(ctx, raw_input);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let mut finish_onboarding = false;
        let mut switch_to_onboarding = false;

        match &mut self.state {
            AndroidState::Onboarding(data) => {
                render_wizard(ui, data);
                self.ui_scale = data.ui_scale;
                if data.finished {
                    finish_onboarding = true;
                }
            }
            AndroidState::Running(app) => {
                let mut scale_delta = 0.0;
                let current_scale = self.ui_scale;

                egui::Panel::top("android_mobile_bar").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🎨 PhotoCraft Android").small().strong());
                        ui.separator();
                        ui.label(egui::RichText::new("Escala:").small());
                        if ui.button("➖").clicked() && current_scale > 0.5 {
                            scale_delta -= 0.05;
                        }
                        ui.label(egui::RichText::new(format!("{:.0}%", current_scale * 100.0)).small());
                        if ui.button("➕").clicked() && current_scale < 1.6 {
                            scale_delta += 0.05;
                        }
                        ui.separator();
                        if ui.button(egui::RichText::new("⚙️ Assistente").small()).clicked() {
                            switch_to_onboarding = true;
                        }
                    });
                });

                if scale_delta != 0.0 {
                    self.ui_scale = (self.ui_scale + scale_delta).clamp(0.5, 1.6);
                }

                app.ui(ui, frame);
            }
        }

        if finish_onboarding {
            let session = self.session.take().unwrap_or_default();
            let services = self.services.take().unwrap_or_default();
            let mut app = PhotocraftApp::new(session, services);
            app.set_theme(&ctx, ThemeKind::Pro);
            if let Some(rs) = self.wgpu_state.clone() {
                app.set_wgpu(rs);
            }
            self.state = AndroidState::Running(Box::new(app));
        } else if switch_to_onboarding {
            let mut data = OnboardingData::default();
            data.ui_scale = self.ui_scale;
            data.storage_granted = true;
            self.state = AndroidState::Onboarding(data);
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
