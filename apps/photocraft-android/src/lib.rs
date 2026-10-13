//! PhotoCraft on Android using eframe's NativeActivity runner.
//!
//! Port maintained by Nepo (@nepotira).
//! Version: 0.1.1-android (based on PhotoCraft upstream v0.8.2).

mod onboarding;

use jni::objects::{JObject, JValue};
use onboarding::{OnboardingAction, OnboardingData, render_wizard};
use photocraft_ui_egui::theme::ThemeKind;
use photocraft_ui_egui::{PhotocraftApp, Services};
use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};
use winit::platform::android::activity::AndroidApp;

static ANDROID_VM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ANDROID_ACTIVITY: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

pub fn request_android_permissions() {
    let vm_ptr = ANDROID_VM.load(Ordering::SeqCst);
    let act_ptr = ANDROID_ACTIVITY.load(Ordering::SeqCst);
    if vm_ptr.is_null() || act_ptr.is_null() {
        log::warn!("Android VM or Activity pointer is not available");
        return;
    }

    unsafe {
        let vm = match jni::JavaVM::from_raw(vm_ptr.cast()) {
            Ok(v) => v,
            Err(e) => {
                log::error!("Failed to create JavaVM: {e}");
                return;
            }
        };

        let mut env = match vm.attach_current_thread() {
            Ok(e) => e,
            Err(e) => {
                log::error!("Failed to attach current thread to JVM: {e}");
                return;
            }
        };

        let activity = JObject::from_raw(act_ptr as jni::sys::jobject);

        let string_class = match env.find_class("java/lang/String") {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to find java/lang/String: {e}");
                return;
            }
        };

        let perms = [
            "android.permission.READ_EXTERNAL_STORAGE",
            "android.permission.WRITE_EXTERNAL_STORAGE",
            "android.permission.READ_MEDIA_IMAGES",
            "android.permission.POST_NOTIFICATIONS",
        ];

        let array = match env.new_object_array(perms.len() as i32, string_class, JObject::null()) {
            Ok(a) => a,
            Err(e) => {
                log::error!("Failed to create String array: {e}");
                return;
            }
        };

        for (i, &p) in perms.iter().enumerate() {
            if let Ok(s) = env.new_string(p) {
                let _ = env.set_object_array_element(&array, i as i32, &s);
            }
        }

        let res = env.call_method(
            &activity,
            "requestPermissions",
            "([Ljava/lang/String;I)V",
            &[
                JValue::Object(&array),
                JValue::Int(1001),
            ],
        );

        if let Err(e) = res {
            log::error!("Failed to call requestPermissions: {e}");
        } else {
            log::info!("requestPermissions called successfully on Android Activity");
        }
    }
}

pub fn open_app_storage_settings() {
    let vm_ptr = ANDROID_VM.load(Ordering::SeqCst);
    let act_ptr = ANDROID_ACTIVITY.load(Ordering::SeqCst);
    if vm_ptr.is_null() || act_ptr.is_null() {
        log::warn!("Android VM or Activity pointer is not available");
        return;
    }

    unsafe {
        let vm = match jni::JavaVM::from_raw(vm_ptr.cast()) {
            Ok(v) => v,
            Err(_) => return,
        };

        let mut env = match vm.attach_current_thread() {
            Ok(e) => e,
            Err(_) => return,
        };

        let activity = JObject::from_raw(act_ptr as jni::sys::jobject);

        let pkg_obj = match env.call_method(&activity, "getPackageName", "()Ljava/lang/String;", &[]) {
            Ok(v) => match v.l() {
                Ok(obj) => obj,
                Err(_) => return,
            },
            Err(_) => return,
        };

        let pkg_jstr = jni::objects::JString::from(pkg_obj);
        let pkg_name: String = match env.get_string(&pkg_jstr) {
            Ok(s) => s.into(),
            Err(_) => "ai.storyteller.photocraft".to_string(),
        };

        let uri_class = match env.find_class("android/net/Uri") {
            Ok(c) => c,
            Err(_) => return,
        };

        let pkg_uri_str = match env.new_string(format!("package:{pkg_name}")) {
            Ok(s) => s,
            Err(_) => return,
        };

        let uri_obj = match env.call_static_method(
            uri_class,
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&pkg_uri_str)],
        ) {
            Ok(v) => match v.l() {
                Ok(obj) => obj,
                Err(_) => return,
            },
            Err(_) => return,
        };

        let intent_class = match env.find_class("android/content/Intent") {
            Ok(c) => c,
            Err(_) => return,
        };

        let action_all_files = match env.new_string("android.settings.MANAGE_APP_ALL_FILES_ACCESS_PERMISSION") {
            Ok(s) => s,
            Err(_) => return,
        };

        let mut intent = env.new_object(
            &intent_class,
            "(Ljava/lang/String;Landroid/net/Uri;)V",
            &[
                JValue::Object(&action_all_files),
                JValue::Object(&uri_obj),
            ],
        );

        if intent.is_err() {
            if let Ok(action_details) = env.new_string("android.settings.APPLICATION_DETAILS_SETTINGS") {
                intent = env.new_object(
                    &intent_class,
                    "(Ljava/lang/String;Landroid/net/Uri;)V",
                    &[
                        JValue::Object(&action_details),
                        JValue::Object(&uri_obj),
                    ],
                );
            }
        }

        if let Ok(intent_obj) = intent {
            let _ = env.call_method(
                &activity,
                "startActivity",
                "(Landroid/content/Intent;)V",
                &[JValue::Object(&intent_obj)],
            );
        }
    }
}

enum AndroidState {
    Onboarding(OnboardingData),
    Running(Box<PhotocraftApp>),
}

struct AndroidShell {
    state: AndroidState,
    zoom_factor: f32,
    last_applied_zoom: f32,
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
            zoom_factor: 1.0,
            last_applied_zoom: 1.0,
            session: Some(session),
            services: Some(services),
            wgpu_state: cc.wgpu_render_state.clone(),
        }
    }
}

impl eframe::App for AndroidShell {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if (self.zoom_factor - self.last_applied_zoom).abs() > 0.001 {
            ctx.set_zoom_factor(self.zoom_factor);
            self.last_applied_zoom = self.zoom_factor;
        }

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
                if let Some(action) = render_wizard(ui, data) {
                    match action {
                        OnboardingAction::RequestSystemPermissions => {
                            request_android_permissions();
                        }
                        OnboardingAction::OpenStorageSettings => {
                            open_app_storage_settings();
                        }
                    }
                }

                self.zoom_factor = data.zoom_factor;
                if data.finished {
                    finish_onboarding = true;
                }
            }
            AndroidState::Running(app) => {
                let mut zoom_delta = 0.0;
                let current_zoom = self.zoom_factor;

                egui::Panel::top("android_mobile_bar").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🎨 PhotoCraft Android").small().strong());
                        ui.separator();
                        ui.label(egui::RichText::new("Zoom:").small());
                        if ui.button("➖").clicked() && current_zoom > 0.6 {
                            zoom_delta -= 0.05;
                        }
                        ui.label(egui::RichText::new(format!("{:.0}%", current_zoom * 100.0)).small());
                        if ui.button("➕").clicked() && current_zoom < 1.5 {
                            zoom_delta += 0.05;
                        }
                        ui.separator();
                        if ui.button(egui::RichText::new("⚙️ Assistente").small()).clicked() {
                            switch_to_onboarding = true;
                        }
                    });
                });

                if zoom_delta != 0.0 {
                    self.zoom_factor = (self.zoom_factor + zoom_delta).clamp(0.6, 1.5);
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
            data.zoom_factor = self.zoom_factor;
            data.permissions_requested = true;
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

    ANDROID_VM.store(app.vm_as_ptr(), Ordering::SeqCst);
    ANDROID_ACTIVITY.store(app.activity_as_ptr(), Ordering::SeqCst);

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
