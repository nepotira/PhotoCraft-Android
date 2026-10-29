//! PhotoCraft on Android using eframe's NativeActivity runner.
//!
//! Port maintained by Nepo (@nepotira).
//! Version: 0.1.4-android (based on PhotoCraft upstream v0.17.0).

mod file_picker;
mod onboarding;

use file_picker::FilePickerState;
use jni::objects::{JObject, JValue};
use onboarding::{OnboardingAction, OnboardingData, render_wizard};
use photocraft_ui_egui::theme::ThemeKind;
use photocraft_ui_egui::{PhotocraftApp, Services};
use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};
use winit::platform::android::activity::AndroidApp;

static ANDROID_VM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ANDROID_ACTIVITY: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

#[inline]
fn clear_jni_exception(env: &mut jni::JNIEnv) {
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
    }
}

pub fn set_immersive_sticky() {
    let vm_ptr = ANDROID_VM.load(Ordering::SeqCst);
    let act_ptr = ANDROID_ACTIVITY.load(Ordering::SeqCst);
    if vm_ptr.is_null() || act_ptr.is_null() {
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

        let window_val = match env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[]) {
            Ok(v) => match v.l() {
                Ok(w) => w,
                Err(_) => {
                    clear_jni_exception(&mut env);
                    return;
                }
            },
            Err(_) => {
                clear_jni_exception(&mut env);
                return;
            }
        };

        let decor_val = match env.call_method(&window_val, "getDecorView", "()Landroid/view/View;", &[]) {
            Ok(v) => match v.l() {
                Ok(d) => d,
                Err(_) => {
                    clear_jni_exception(&mut env);
                    return;
                }
            },
            Err(_) => {
                clear_jni_exception(&mut env);
                return;
            }
        };

        // Combined flags for IMMERSIVE_STICKY fullscreen:
        // LAYOUT_STABLE(256) | LAYOUT_HIDE_NAV(512) | LAYOUT_FULLSCREEN(1024)
        // | HIDE_NAV(2) | FULLSCREEN(4) | IMMERSIVE_STICKY(4096) = 5894
        let flags: i32 = 5894;

        let _ = env.call_method(&decor_val, "setSystemUiVisibility", "(I)V", &[JValue::Int(flags)]);
        clear_jni_exception(&mut env);

        let _ = env.call_method(&window_val, "addFlags", "(I)V", &[JValue::Int(1024)]);
        clear_jni_exception(&mut env);
    }
}

pub fn has_storage_permission() -> bool {
    let vm_ptr = ANDROID_VM.load(Ordering::SeqCst);
    let act_ptr = ANDROID_ACTIVITY.load(Ordering::SeqCst);
    if vm_ptr.is_null() || act_ptr.is_null() {
        return false;
    }

    unsafe {
        let vm = match jni::JavaVM::from_raw(vm_ptr.cast()) {
            Ok(v) => v,
            Err(_) => return false,
        };

        let mut env = match vm.attach_current_thread() {
            Ok(e) => e,
            Err(_) => return false,
        };

        let activity = JObject::from_raw(act_ptr as jni::sys::jobject);

        // Check Environment.isExternalStorageManager() on Android 11+
        if let Ok(env_class) = env.find_class("android/os/Environment") {
            if let Ok(res) = env.call_static_method(env_class, "isExternalStorageManager", "()Z", &[]) {
                if let Ok(true) = res.z() {
                    return true;
                }
            }
        }
        clear_jni_exception(&mut env);

        // Check READ_EXTERNAL_STORAGE
        if let Ok(perm_str) = env.new_string("android.permission.READ_EXTERNAL_STORAGE") {
            if let Ok(res) = env.call_method(&activity, "checkSelfPermission", "(Ljava/lang/String;)I", &[JValue::Object(&perm_str)]) {
                if let Ok(0) = res.i() {
                    return true;
                }
            }
        }
        clear_jni_exception(&mut env);

        // Check READ_MEDIA_IMAGES (Android 13+)
        if let Ok(perm_str) = env.new_string("android.permission.READ_MEDIA_IMAGES") {
            if let Ok(res) = env.call_method(&activity, "checkSelfPermission", "(Ljava/lang/String;)I", &[JValue::Object(&perm_str)]) {
                if let Ok(0) = res.i() {
                    return true;
                }
            }
        }
        clear_jni_exception(&mut env);

        false
    }
}

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
                clear_jni_exception(&mut env);
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
                clear_jni_exception(&mut env);
                log::error!("Failed to create String array: {e}");
                return;
            }
        };

        for (i, &p) in perms.iter().enumerate() {
            if let Ok(s) = env.new_string(p) {
                let _ = env.set_object_array_element(&array, i as i32, &s);
            }
            clear_jni_exception(&mut env);
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
            clear_jni_exception(&mut env);
            log::error!("Failed to call requestPermissions: {e}");
        } else {
            clear_jni_exception(&mut env);
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
                Err(_) => {
                    clear_jni_exception(&mut env);
                    return;
                }
            },
            Err(_) => {
                clear_jni_exception(&mut env);
                return;
            }
        };

        let pkg_jstr = jni::objects::JString::from(pkg_obj);
        let pkg_name: String = match env.get_string(&pkg_jstr) {
            Ok(s) => s.into(),
            Err(_) => {
                clear_jni_exception(&mut env);
                "ai.storyteller.photocraft".to_string()
            }
        };

        let uri_class = match env.find_class("android/net/Uri") {
            Ok(c) => c,
            Err(_) => {
                clear_jni_exception(&mut env);
                return;
            }
        };

        let pkg_uri_str = match env.new_string(format!("package:{pkg_name}")) {
            Ok(s) => s,
            Err(_) => {
                clear_jni_exception(&mut env);
                return;
            }
        };

        let uri_obj = match env.call_static_method(
            uri_class,
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&pkg_uri_str)],
        ) {
            Ok(v) => match v.l() {
                Ok(obj) => obj,
                Err(_) => {
                    clear_jni_exception(&mut env);
                    return;
                }
            },
            Err(_) => {
                clear_jni_exception(&mut env);
                return;
            }
        };

        let intent_class = match env.find_class("android/content/Intent") {
            Ok(c) => c,
            Err(_) => {
                clear_jni_exception(&mut env);
                return;
            }
        };

        let action_all_files = match env.new_string("android.settings.MANAGE_APP_ALL_FILES_ACCESS_PERMISSION") {
            Ok(s) => s,
            Err(_) => {
                clear_jni_exception(&mut env);
                return;
            }
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
            clear_jni_exception(&mut env);
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
        clear_jni_exception(&mut env);
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
    file_picker: FilePickerState,
    wgpu_state: Option<eframe::egui_wgpu::RenderState>,
    frames_since_immersive: u32,
}

impl AndroidShell {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        PhotocraftApp::setup_context(&cc.egui_ctx, ThemeKind::Pro);
        cc.egui_ctx.global_style_mut(|s| {
            s.spacing.interact_size = egui::vec2(32.0, 32.0);
            s.interaction.interact_radius = 8.0;
        });

        let session = photocraft_engine::Session::default();
        let mut services = Services::default();
        let file_picker = FilePickerState::new();

        let fp_clone = file_picker.clone();
        services.file_dialog = Some(Box::new(move |request, _frame, reply| {
            fp_clone.open_dialog(request, reply);
        }));

        set_immersive_sticky();

        Self {
            state: AndroidState::Onboarding(OnboardingData::default()),
            zoom_factor: 1.0,
            last_applied_zoom: 1.0,
            session: Some(session),
            services: Some(services),
            file_picker,
            wgpu_state: cc.wgpu_render_state.clone(),
            frames_since_immersive: 0,
        }
    }
}

impl eframe::App for AndroidShell {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if (self.zoom_factor - self.last_applied_zoom).abs() > 0.001 {
            ctx.set_zoom_factor(self.zoom_factor);
            self.last_applied_zoom = self.zoom_factor;
        }

        self.frames_since_immersive += 1;
        if self.frames_since_immersive == 10 || self.frames_since_immersive == 120 {
            set_immersive_sticky();
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

        let storage_granted = has_storage_permission();

        match &mut self.state {
            AndroidState::Onboarding(data) => {
                if let Some(action) = render_wizard(ui, data, storage_granted) {
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
                        ui.label(egui::RichText::new("PhotoCraft Android").small().strong());
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
                        if ui.button(egui::RichText::new("Assistente").small()).clicked() {
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

        // Always show in-app file picker modal if an Open or Save request is active
        self.file_picker.ui(&ctx);

        if finish_onboarding {
            let session = self.session.take().unwrap_or_default();
            let services = self.services.take().unwrap_or_default();
            let mut app = PhotocraftApp::new(session, services);
            app.set_theme(&ctx, ThemeKind::Pro);
            if let Some(rs) = self.wgpu_state.clone() {
                app.set_wgpu(rs);
            }
            self.state = AndroidState::Running(Box::new(app));
            set_immersive_sticky();
        } else if switch_to_onboarding {
            let mut data = OnboardingData::default();
            data.zoom_factor = self.zoom_factor;
            data.permissions_requested = storage_granted;
            self.state = AndroidState::Onboarding(data);
            set_immersive_sticky();
        }
    }
}

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    std::panic::set_hook(Box::new(|info| {
        log::error!("CRITICAL PANIC in PhotoCraft Android: {info}");
    }));

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    log::info!("PhotoCraft Android & Googlebook OS v0.1.4 starting up");

    ANDROID_VM.store(app.vm_as_ptr(), Ordering::SeqCst);
    ANDROID_ACTIVITY.store(app.activity_as_ptr(), Ordering::SeqCst);

    let mut options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };

    // Native GPU support for ARM64 & x86_64 Intel UHD Graphics (Chromebook Jinlon / Googlebook OS)
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(create) = &mut options.wgpu_options.wgpu_setup {
        // Dual backend: Vulkan primary, fallback to OpenGL ES (GLES via EGL)
        create.instance_descriptor.backends = eframe::wgpu::Backends::VULKAN | eframe::wgpu::Backends::GL;
        // Skip indirect-call validation shader compiler crashes on Intel GPUs (Comet Lake / Gen 9.5)
        create.instance_descriptor.flags = create.instance_descriptor.flags
            .difference(eframe::wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL)
            .with_env();
        create.power_preference = eframe::wgpu::PowerPreference::HighPerformance;
    }
    photocraft_ui_egui::gpu_canvas::use_adapter_limits(&mut options.wgpu_options.wgpu_setup);

    let result = eframe::run_native(
        "PhotoCraft",
        options,
        Box::new(|cc| Ok(Box::new(AndroidShell::new(cc)))),
    );

    if let Err(e) = result {
        log::error!("PhotoCraft exited with error: {e}");
    }
}
