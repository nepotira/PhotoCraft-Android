//! PhotoCraft on Android & Googlebook OS using eframe's NativeActivity runner.
//!
//! Port maintained by Nepo (@nepotira).
//! Version: 0.1.5-android (based on PhotoCraft upstream v0.17.0).

mod file_picker;
mod onboarding;

use file_picker::FilePickerState;
use jni::objects::{JClass, JObject, JString, JValue};
use onboarding::{OnboardingAction, OnboardingData, render_wizard};
use photocraft_ui_egui::theme::ThemeKind;
use photocraft_ui_egui::{PhotocraftApp, Services};
use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use winit::platform::android::activity::AndroidApp;

static ANDROID_VM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ANDROID_ACTIVITY: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static DROP_HELPER_CLASS: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static DROP_HELPER_INITIALIZED: AtomicBool = AtomicBool::new(false);
static PENDING_FILES: Mutex<Vec<(String, Vec<u8>)>> = Mutex::new(Vec::new());

#[inline]
fn clear_jni_exception(env: &mut jni::JNIEnv) {
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
    }
}

pub fn enqueue_pending_file(name: String, bytes: Vec<u8>) {
    if let Ok(mut lock) = PENDING_FILES.lock() {
        lock.push((name, bytes));
    }
}

pub fn take_pending_files() -> Vec<(String, Vec<u8>)> {
    if let Ok(mut lock) = PENDING_FILES.lock() {
        std::mem::take(&mut *lock)
    } else {
        Vec::new()
    }
}

/// Native JNI callback invoked by DropHelper.java when files are dropped.
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_photocraft_DropHelper_onDrop(
    mut env: jni::JNIEnv,
    _class: JClass,
    uri_obj: JString,
) {
    if let Ok(uri_str) = env.get_string(&uri_obj) {
        let uri_s: String = uri_str.into();
        log::info!("DropHelper onDrop received URI: {uri_s}");
        if let Some((name, bytes)) = read_content_or_file_uri(&uri_s) {
            enqueue_pending_file(name, bytes);
        }
    }
}

/// Reads content bytes and resolves file name from a content:// or file:// URI.
pub fn read_content_or_file_uri(uri_str: &str) -> Option<(String, Vec<u8>)> {
    let vm_ptr = ANDROID_VM.load(Ordering::SeqCst);
    let act_ptr = ANDROID_ACTIVITY.load(Ordering::SeqCst);
    if vm_ptr.is_null() || act_ptr.is_null() {
        return None;
    }

    unsafe {
        let vm = match jni::JavaVM::from_raw(vm_ptr.cast()) {
            Ok(v) => v,
            Err(_) => return None,
        };
        let mut env = match vm.attach_current_thread() {
            Ok(e) => e,
            Err(_) => return None,
        };
        let activity = JObject::from_raw(act_ptr as jni::sys::jobject);

        // If it starts with file:// or a local path:
        let local_path = if uri_str.starts_with("file://") {
            Some(&uri_str[7..])
        } else if uri_str.starts_with('/') {
            Some(uri_str)
        } else {
            None
        };

        if let Some(path_str) = local_path {
            let path = std::path::Path::new(path_str);
            if let Ok(bytes) = std::fs::read(path) {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "document.psd".to_string());
                return Some((name, bytes));
            }
        }

        let uri_class = match env.find_class("android/net/Uri") {
            Ok(c) => c,
            Err(_) => {
                clear_jni_exception(&mut env);
                return None;
            }
        };

        let uri_jstr = match env.new_string(uri_str) {
            Ok(s) => s,
            Err(_) => {
                clear_jni_exception(&mut env);
                return None;
            }
        };

        let uri_obj = match env.call_static_method(
            uri_class,
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&uri_jstr)],
        ) {
            Ok(v) => match v.l() {
                Ok(u) => u,
                Err(_) => {
                    clear_jni_exception(&mut env);
                    return None;
                }
            },
            Err(_) => {
                clear_jni_exception(&mut env);
                return None;
            }
        };

        let resolver = match env.call_method(&activity, "getContentResolver", "()Landroid/content/ContentResolver;", &[]) {
            Ok(v) => match v.l() {
                Ok(r) => r,
                Err(_) => {
                    clear_jni_exception(&mut env);
                    return None;
                }
            },
            Err(_) => {
                clear_jni_exception(&mut env);
                return None;
            }
        };

        // Query OpenableColumns.DISPLAY_NAME ("_display_name")
        let mut file_name = String::new();
        if let Ok(col_name) = env.new_string("_display_name") {
            if let Ok(string_cls) = env.find_class("java/lang/String") {
                if let Ok(proj_array) = env.new_object_array(1, string_cls, &col_name) {
                    let null_obj = JObject::null();
                    let cursor_res = env.call_method(
                        &resolver,
                        "query",
                        "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
                        &[
                            JValue::Object(&uri_obj),
                            JValue::Object(&proj_array),
                            JValue::Object(&null_obj),
                            JValue::Object(&null_obj),
                            JValue::Object(&null_obj),
                        ],
                    );
                    if let Ok(c_val) = cursor_res {
                        if let Ok(c) = c_val.l() {
                            if !c.is_null() {
                                if let Ok(first) = env.call_method(&c, "moveToFirst", "()Z", &[]) {
                                    if first.z().unwrap_or(false) {
                                        if let Ok(idx_val) = env.call_method(&c, "getColumnIndex", "(Ljava/lang/String;)I", &[JValue::Object(&col_name)]) {
                                            let col_idx = idx_val.i().unwrap_or(0);
                                            if col_idx >= 0 {
                                                if let Ok(str_val) = env.call_method(&c, "getString", "(I)Ljava/lang/String;", &[JValue::Int(col_idx)]) {
                                                    if let Ok(name_jstr) = str_val.l() {
                                                        if !name_jstr.is_null() {
                                                            if let Ok(s) = env.get_string(&JString::from(name_jstr)) {
                                                                file_name = s.into();
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                let _ = env.call_method(&c, "close", "()V", &[]);
                                clear_jni_exception(&mut env);
                            }
                        }
                    }
                }
            }
        }
        clear_jni_exception(&mut env);

        if file_name.is_empty() {
            if let Ok(seg) = env.call_method(&uri_obj, "getLastPathSegment", "()Ljava/lang/String;", &[]) {
                if let Ok(s) = seg.l() {
                    if !s.is_null() {
                        if let Ok(n) = env.get_string(&JString::from(s)) {
                            file_name = n.into();
                        }
                    }
                }
            }
        }
        if file_name.is_empty() {
            file_name = "document.psd".to_string();
        }

        // Open InputStream and read bytes
        let is_val = match env.call_method(
            &resolver,
            "openInputStream",
            "(Landroid/net/Uri;)Ljava/io/InputStream;",
            &[JValue::Object(&uri_obj)],
        ) {
            Ok(v) => match v.l() {
                Ok(stream) => stream,
                Err(_) => {
                    clear_jni_exception(&mut env);
                    return None;
                }
            },
            Err(_) => {
                clear_jni_exception(&mut env);
                return None;
            }
        };

        if is_val.is_null() {
            return None;
        }

        let byte_buf = match env.new_byte_array(64 * 1024) {
            Ok(b) => b,
            Err(_) => {
                clear_jni_exception(&mut env);
                return None;
            }
        };

        let mut all_bytes = Vec::new();
        loop {
            let read_res = match env.call_method(&is_val, "read", "([B)I", &[JValue::Object(&byte_buf)]) {
                Ok(v) => v.i().unwrap_or(-1),
                Err(_) => {
                    clear_jni_exception(&mut env);
                    break;
                }
            };
            if read_res <= 0 {
                break;
            }
            let mut chunk = vec![0u8; read_res as usize];
            let raw_chunk = std::slice::from_raw_parts_mut(chunk.as_mut_ptr() as *mut i8, read_res as usize);
            let _ = env.get_byte_array_region(&byte_buf, 0, raw_chunk);
            all_bytes.extend_from_slice(&chunk);
        }
        let _ = env.call_method(&is_val, "close", "()V", &[]);
        clear_jni_exception(&mut env);

        Some((file_name, all_bytes))
    }
}

/// Checks if activity.getIntent() contains a content or file URI (e.g. from "Open With"),
/// and clears it so it is not processed repeatedly.
pub fn check_and_clear_activity_intent() -> Option<String> {
    let vm_ptr = ANDROID_VM.load(Ordering::SeqCst);
    let act_ptr = ANDROID_ACTIVITY.load(Ordering::SeqCst);
    if vm_ptr.is_null() || act_ptr.is_null() {
        return None;
    }

    unsafe {
        let vm = match jni::JavaVM::from_raw(vm_ptr.cast()) {
            Ok(v) => v,
            Err(_) => return None,
        };
        let mut env = match vm.attach_current_thread() {
            Ok(e) => e,
            Err(_) => return None,
        };
        let activity = JObject::from_raw(act_ptr as jni::sys::jobject);

        let intent_val = match env.call_method(&activity, "getIntent", "()Landroid/content/Intent;", &[]) {
            Ok(v) => match v.l() {
                Ok(i) => i,
                Err(_) => {
                    clear_jni_exception(&mut env);
                    return None;
                }
            },
            Err(_) => {
                clear_jni_exception(&mut env);
                return None;
            }
        };

        if intent_val.is_null() {
            return None;
        }

        // Check intent.getData()
        if let Ok(uri_val) = env.call_method(&intent_val, "getData", "()Landroid/net/Uri;", &[]) {
            if let Ok(uri) = uri_val.l() {
                if !uri.is_null() {
                    if let Ok(str_obj) = env.call_method(&uri, "toString", "()Ljava/lang/String;", &[]) {
                        if let Ok(s) = str_obj.l() {
                            if !s.is_null() {
                                if let Ok(jstr) = env.get_string(&JString::from(s)) {
                                    let result: String = jstr.into();
                                    let null_obj = JObject::null();
                                    let _ = env.call_method(&intent_val, "setData", "(Landroid/net/Uri;)Landroid/content/Intent;", &[JValue::Object(&null_obj)]);
                                    clear_jni_exception(&mut env);
                                    if !result.is_empty() {
                                        return Some(result);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        clear_jni_exception(&mut env);

        // Check intent.getClipData()
        if let Ok(clip_val) = env.call_method(&intent_val, "getClipData", "()Landroid/content/ClipData;", &[]) {
            if let Ok(clip) = clip_val.l() {
                if !clip.is_null() {
                    let count = env.call_method(&clip, "getItemCount", "()I", &[]).map(|v| v.i().unwrap_or(0)).unwrap_or(0);
                    if count > 0 {
                        if let Ok(item_val) = env.call_method(&clip, "getItemAt", "(I)Landroid/content/ClipData$Item;", &[JValue::Int(0)]) {
                            if let Ok(item) = item_val.l() {
                                if !item.is_null() {
                                    if let Ok(uri_val) = env.call_method(&item, "getUri", "()Landroid/net/Uri;", &[]) {
                                        if let Ok(uri) = uri_val.l() {
                                            if !uri.is_null() {
                                                if let Ok(str_obj) = env.call_method(&uri, "toString", "()Ljava/lang/String;", &[]) {
                                                    if let Ok(s) = str_obj.l() {
                                                        if !s.is_null() {
                                                            if let Ok(jstr) = env.get_string(&JString::from(s)) {
                                                                let result: String = jstr.into();
                                                                let null_obj = JObject::null();
                                                                let _ = env.call_method(&intent_val, "setClipData", "(Landroid/content/ClipData;)V", &[JValue::Object(&null_obj)]);
                                                                clear_jni_exception(&mut env);
                                                                if !result.is_empty() {
                                                                    return Some(result);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        clear_jni_exception(&mut env);

        None
    }
}

/// Initializes DropHelper via direct ClassLoader or InMemoryDexClassLoader from assets/classes.dex.
pub fn init_drop_helper_if_needed() {
    if DROP_HELPER_INITIALIZED.load(Ordering::SeqCst) {
        return;
    }

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

        // 1. Check if DropHelper is directly available (e.g. from primary APK classloader)
        if let Ok(helper_cls) = env.find_class("ai/storyteller/photocraft/DropHelper") {
            log::info!("DropHelper class found directly in primary ClassLoader");
            if let Ok(global_ref) = env.new_global_ref(&helper_cls) {
                let raw_ptr = global_ref.as_raw();
                std::mem::forget(global_ref);
                DROP_HELPER_CLASS.store(raw_ptr as *mut c_void, Ordering::SeqCst);
            }
            let _ = env.call_static_method(
                &helper_cls,
                "attach",
                "(Landroid/app/Activity;)V",
                &[JValue::Object(&activity)],
            );
            clear_jni_exception(&mut env);
            DROP_HELPER_INITIALIZED.store(true, Ordering::SeqCst);
            return;
        }
        clear_jni_exception(&mut env);

        // 2. Load classes.dex from assets using AssetManager and InMemoryDexClassLoader
        let asset_mgr = match env.call_method(&activity, "getAssets", "()Landroid/content/res/AssetManager;", &[]) {
            Ok(v) => match v.l() {
                Ok(am) => am,
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

        let dex_name = match env.new_string("classes.dex") {
            Ok(s) => s,
            Err(_) => {
                clear_jni_exception(&mut env);
                return;
            }
        };

        let is_val = match env.call_method(&asset_mgr, "open", "(Ljava/lang/String;)Ljava/io/InputStream;", &[JValue::Object(&dex_name)]) {
            Ok(v) => match v.l() {
                Ok(s) => s,
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

        if is_val.is_null() {
            return;
        }

        let mut dex_bytes = Vec::new();
        let buf = match env.new_byte_array(16 * 1024) {
            Ok(b) => b,
            Err(_) => {
                clear_jni_exception(&mut env);
                return;
            }
        };

        loop {
            let read_count = match env.call_method(&is_val, "read", "([B)I", &[JValue::Object(&buf)]) {
                Ok(v) => v.i().unwrap_or(-1),
                Err(_) => {
                    clear_jni_exception(&mut env);
                    break;
                }
            };
            if read_count <= 0 {
                break;
            }
            let mut chunk = vec![0u8; read_count as usize];
            let raw_chunk = std::slice::from_raw_parts_mut(chunk.as_mut_ptr() as *mut i8, read_count as usize);
            let _ = env.get_byte_array_region(&buf, 0, raw_chunk);
            dex_bytes.extend_from_slice(&chunk);
        }
        let _ = env.call_method(&is_val, "close", "()V", &[]);
        clear_jni_exception(&mut env);

        if dex_bytes.is_empty() {
            log::warn!("classes.dex in assets was empty or not found");
            return;
        }

        log::info!("Loaded {} bytes of classes.dex from assets", dex_bytes.len());
        let leaked_dex = Box::leak(dex_bytes.into_boxed_slice());
        let byte_buffer = match env.new_direct_byte_buffer(leaked_dex.as_mut_ptr(), leaked_dex.len()) {
            Ok(bb) => bb,
            Err(e) => {
                clear_jni_exception(&mut env);
                log::error!("Failed to create direct ByteBuffer: {e}");
                return;
            }
        };

        let class_loader = match env.call_method(&activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[]) {
            Ok(v) => match v.l() {
                Ok(cl) => cl,
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

        let in_memory_loader_cls = match env.find_class("dalvik/system/InMemoryDexClassLoader") {
            Ok(c) => c,
            Err(e) => {
                clear_jni_exception(&mut env);
                log::error!("Failed to find InMemoryDexClassLoader: {e}");
                return;
            }
        };

        let loader = match env.new_object(
            &in_memory_loader_cls,
            "(Ljava/nio/ByteBuffer;Ljava/lang/ClassLoader;)V",
            &[JValue::Object(&byte_buffer), JValue::Object(&class_loader)],
        ) {
            Ok(l) => l,
            Err(e) => {
                clear_jni_exception(&mut env);
                log::error!("Failed to instantiate InMemoryDexClassLoader: {e}");
                return;
            }
        };

        let class_name = match env.new_string("ai.storyteller.photocraft.DropHelper") {
            Ok(s) => s,
            Err(_) => {
                clear_jni_exception(&mut env);
                return;
            }
        };

        let loaded_class_obj = match env.call_method(
            &loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&class_name)],
        ) {
            Ok(v) => match v.l() {
                Ok(c) => c,
                Err(_) => {
                    clear_jni_exception(&mut env);
                    return;
                }
            },
            Err(e) => {
                clear_jni_exception(&mut env);
                log::error!("Failed to loadClass ai.storyteller.photocraft.DropHelper: {e}");
                return;
            }
        };

        let loaded_class: JClass = loaded_class_obj.into();
        if let Ok(global_ref) = env.new_global_ref(&loaded_class) {
            let raw_ptr = global_ref.as_raw();
            std::mem::forget(global_ref);
            DROP_HELPER_CLASS.store(raw_ptr as *mut c_void, Ordering::SeqCst);
        }

        let _ = env.call_static_method(
            &loaded_class,
            "attach",
            "(Landroid/app/Activity;)V",
            &[JValue::Object(&activity)],
        );
        clear_jni_exception(&mut env);

        DROP_HELPER_INITIALIZED.store(true, Ordering::SeqCst);
        log::info!("DropHelper attached successfully via InMemoryDexClassLoader!");
    }
}

/// Polls dropped files from DropHelper's static thread-safe queue.
pub fn poll_drop_helper() {
    let vm_ptr = ANDROID_VM.load(Ordering::SeqCst);
    let class_ptr = DROP_HELPER_CLASS.load(Ordering::SeqCst);
    if vm_ptr.is_null() || class_ptr.is_null() {
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
        let helper_cls = JClass::from_raw(class_ptr as jni::sys::jclass);

        loop {
            let res = env.call_static_method(
                &helper_cls,
                "pollDrop",
                "()Ljava/lang/String;",
                &[],
            );
            match res {
                Ok(val) => match val.l() {
                    Ok(obj) if !obj.is_null() => {
                        if let Ok(jstr) = env.get_string(&JString::from(obj)) {
                            let uri_s: String = jstr.into();
                            log::info!("Polled dropped URI from DropHelper: {uri_s}");
                            if let Some((name, bytes)) = read_content_or_file_uri(&uri_s) {
                                enqueue_pending_file(name, bytes);
                            }
                        }
                    }
                    _ => break,
                },
                Err(_) => {
                    clear_jni_exception(&mut env);
                    break;
                }
            }
        }
        clear_jni_exception(&mut env);
    }
}

/// Changes the system mouse hover cursor icon on Googlebook OS / Android 7.0+.
pub fn set_android_pointer_icon(icon: egui::CursorIcon) {
    let vm_ptr = ANDROID_VM.load(Ordering::SeqCst);
    let act_ptr = ANDROID_ACTIVITY.load(Ordering::SeqCst);
    let class_ptr = DROP_HELPER_CLASS.load(Ordering::SeqCst);
    if vm_ptr.is_null() || act_ptr.is_null() {
        return;
    }

    let pointer_type: i32 = match icon {
        egui::CursorIcon::Default => 1000, // TYPE_ARROW
        egui::CursorIcon::PointingHand => 1002, // TYPE_HAND
        egui::CursorIcon::Text | egui::CursorIcon::VerticalText => 1008, // TYPE_TEXT
        egui::CursorIcon::Crosshair => 1007, // TYPE_CROSSHAIR
        egui::CursorIcon::ResizeHorizontal
        | egui::CursorIcon::ResizeEast
        | egui::CursorIcon::ResizeWest
        | egui::CursorIcon::ResizeColumn => 1014, // TYPE_HORIZONTAL_DOUBLE_ARROW
        egui::CursorIcon::ResizeVertical
        | egui::CursorIcon::ResizeNorth
        | egui::CursorIcon::ResizeSouth
        | egui::CursorIcon::ResizeRow => 1015, // TYPE_VERTICAL_DOUBLE_ARROW
        egui::CursorIcon::ResizeNorthEast
        | egui::CursorIcon::ResizeSouthWest
        | egui::CursorIcon::ResizeNeSw => 1016, // TYPE_TOP_RIGHT_DIAGONAL_DOUBLE_ARROW
        egui::CursorIcon::ResizeNorthWest
        | egui::CursorIcon::ResizeSouthEast
        | egui::CursorIcon::ResizeNwSe => 1017, // TYPE_TOP_LEFT_DIAGONAL_DOUBLE_ARROW
        egui::CursorIcon::Grab => 1020, // TYPE_GRAB
        egui::CursorIcon::Grabbing => 1021, // TYPE_GRABBING
        egui::CursorIcon::NotAllowed | egui::CursorIcon::NoDrop => 1012, // TYPE_NO_DROP
        egui::CursorIcon::Wait | egui::CursorIcon::Progress => 1004, // TYPE_WAIT
        egui::CursorIcon::Help => 1003, // TYPE_HELP
        egui::CursorIcon::Copy => 1011, // TYPE_COPY
        egui::CursorIcon::ZoomIn => 1018, // TYPE_ZOOM_IN
        egui::CursorIcon::ZoomOut => 1019, // TYPE_ZOOM_OUT
        egui::CursorIcon::AllScroll | egui::CursorIcon::Move => 1013, // TYPE_ALL_SCROLL
        _ => 1000,
    };

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

        // Try calling DropHelper.setPointerIcon(activity, pointer_type) using global class ref
        if !class_ptr.is_null() {
            let helper_cls = JClass::from_raw(class_ptr as jni::sys::jclass);
            let res = env.call_static_method(
                &helper_cls,
                "setPointerIcon",
                "(Landroid/app/Activity;I)V",
                &[JValue::Object(&activity), JValue::Int(pointer_type)],
            );
            if res.is_ok() {
                clear_jni_exception(&mut env);
                return;
            }
        }
        clear_jni_exception(&mut env);

        // Fallback: check if class is in primary ClassLoader
        if let Ok(helper_cls) = env.find_class("ai/storyteller/photocraft/DropHelper") {
            let res = env.call_static_method(
                &helper_cls,
                "setPointerIcon",
                "(Landroid/app/Activity;I)V",
                &[JValue::Object(&activity), JValue::Int(pointer_type)],
            );
            if res.is_ok() {
                clear_jni_exception(&mut env);
                return;
            }
        }
        clear_jni_exception(&mut env);
    }
}

/// Queries the top window caption / status bar inset in physical pixels.
pub fn get_window_caption_top_inset() -> f32 {
    let vm_ptr = ANDROID_VM.load(Ordering::SeqCst);
    let act_ptr = ANDROID_ACTIVITY.load(Ordering::SeqCst);
    let class_ptr = DROP_HELPER_CLASS.load(Ordering::SeqCst);
    if vm_ptr.is_null() || act_ptr.is_null() {
        return 0.0;
    }

    unsafe {
        let vm = match jni::JavaVM::from_raw(vm_ptr.cast()) {
            Ok(v) => v,
            Err(_) => return 0.0,
        };
        let mut env = match vm.attach_current_thread() {
            Ok(e) => e,
            Err(_) => return 0.0,
        };
        let activity = JObject::from_raw(act_ptr as jni::sys::jobject);

        if !class_ptr.is_null() {
            let helper_cls = JClass::from_raw(class_ptr as jni::sys::jclass);
            if let Ok(res) = env.call_static_method(
                &helper_cls,
                "getCaptionTopInset",
                "(Landroid/app/Activity;)I",
                &[JValue::Object(&activity)],
            ) {
                let inset = res.i().unwrap_or(0);
                if inset > 0 {
                    clear_jni_exception(&mut env);
                    return inset as f32;
                }
            }
        }
        clear_jni_exception(&mut env);

        if let Ok(helper_cls) = env.find_class("ai/storyteller/photocraft/DropHelper") {
            if let Ok(res) = env.call_static_method(
                &helper_cls,
                "getCaptionTopInset",
                "(Landroid/app/Activity;)I",
                &[JValue::Object(&activity)],
            ) {
                let inset = res.i().unwrap_or(0);
                if inset > 0 {
                    clear_jni_exception(&mut env);
                    return inset as f32;
                }
            }
        }
        clear_jni_exception(&mut env);

        // Fallback: query decorView.getRootWindowInsets()
        if let Ok(window_val) = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[]) {
            if let Ok(w) = window_val.l() {
                if let Ok(decor_val) = env.call_method(&w, "getDecorView", "()Landroid/view/View;", &[]) {
                    if let Ok(d) = decor_val.l() {
                        if let Ok(insets_val) = env.call_method(&d, "getRootWindowInsets", "()Landroid/view/WindowInsets;", &[]) {
                            if let Ok(ins) = insets_val.l() {
                                if !ins.is_null() {
                                    // WindowInsets.Type.captionBar() = 4 | statusBars() = 1 => 5
                                    if let Ok(insets_res) = env.call_method(&ins, "getInsets", "(I)Landroid/graphics/Insets;", &[JValue::Int(5)]) {
                                        if let Ok(insets_obj) = insets_res.l() {
                                            if !insets_obj.is_null() {
                                                if let Ok(top) = env.get_field(&insets_obj, "top", "I") {
                                                    let top_i = top.i().unwrap_or(0);
                                                    clear_jni_exception(&mut env);
                                                    return top_i as f32;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        clear_jni_exception(&mut env);
        0.0
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

        let pkg_jstr = JString::from(pkg_obj);
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
    frames_since_start: u32,
    last_cursor_icon: egui::CursorIcon,
    caption_top_inset_points: f32,
    manual_top_offset: f32,
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

        services.import = Some(Box::new(|name: &str, bytes: &[u8], opts: &photocraft_io::ImportOptions| {
            photocraft_io::import_with_options(name, bytes, opts)
                .map(|r| (r.document, r.warnings))
                .map_err(|e| e.to_string())
        }));

        services.export = Some(Box::new(|doc: &photocraft_doc::Document, path: &str, settings: &photocraft_ui_egui::ExportSettings| {
            let opts = settings.io_options();
            photocraft_io::export(doc, path, &opts)
                .map(|r| (r.bytes, r.warnings))
                .map_err(|e| e.to_string())
        }));

        services.save_file = Some(std::sync::Arc::new(|doc: &photocraft_doc::Document, path: &str, settings: &photocraft_ui_egui::ExportSettings, ctl: &photocraft_ui_egui::jobs_ui::SaveCtl| {
            ctl.progress(0.1, "Encoding");
            let opts = settings.io_options();
            let r = photocraft_io::export(doc, path, &opts).map_err(|e| e.to_string())?;
            ctl.commit()?;
            std::fs::write(path, &r.bytes).map_err(|e| e.to_string())?;
            Ok(r.warnings)
        }));

        services.write = Some(Box::new(|path: &str, bytes: &[u8]| {
            std::fs::write(path, bytes).map_err(|e| e.to_string())
        }));

        services.file_stamp = Some(Box::new(|path: &str| {
            let meta = std::fs::metadata(path).ok()?;
            Some(photocraft_ui_egui::FileStamp {
                len: meta.len(),
                modified: meta.modified().ok(),
            })
        }));

        services.same_file = Some(Box::new(|a: &str, b: &str| {
            a == b || std::fs::canonicalize(a).ok() == std::fs::canonicalize(b).ok()
        }));

        let fp_clone = file_picker.clone();
        services.file_dialog = Some(Box::new(move |request, _frame, reply| {
            fp_clone.open_dialog(request, reply);
        }));

        Self {
            state: AndroidState::Onboarding(OnboardingData::default()),
            zoom_factor: 1.0,
            last_applied_zoom: 1.0,
            session: Some(session),
            services: Some(services),
            file_picker,
            wgpu_state: cc.wgpu_render_state.clone(),
            frames_since_start: 0,
            last_cursor_icon: egui::CursorIcon::Default,
            caption_top_inset_points: 0.0,
            manual_top_offset: 0.0,
        }
    }
}

impl eframe::App for AndroidShell {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if (self.zoom_factor - self.last_applied_zoom).abs() > 0.001 {
            ctx.set_zoom_factor(self.zoom_factor);
            self.last_applied_zoom = self.zoom_factor;
        }

        self.frames_since_start += 1;
        if self.frames_since_start == 1 || self.frames_since_start == 5 || self.frames_since_start == 30 || self.frames_since_start == 120 {
            init_drop_helper_if_needed();
            let top_px = get_window_caption_top_inset();
            if top_px > 0.0 {
                self.caption_top_inset_points = top_px / ctx.pixels_per_point();
            }
        }

        // Poll dropped files from DropHelper (Google Drive & local Downloads drag & drop)
        poll_drop_helper();

        // Bug 1: Mouse Hover / Cursor Icon changes on Googlebook OS & Android desktop
        let current_cursor = ctx.output(|o| o.cursor_icon);
        if current_cursor != self.last_cursor_icon {
            set_android_pointer_icon(current_cursor);
            self.last_cursor_icon = current_cursor;
        }

        // Bug 3: Check for pending file from Android Intent (e.g. clicking .psd file in Files/Drive)
        if self.frames_since_start % 20 == 0 || self.frames_since_start == 2 {
            if let Some(uri_str) = check_and_clear_activity_intent() {
                log::info!("Discovered file from Android Intent: {uri_str}");
                if let Some((name, bytes)) = read_content_or_file_uri(&uri_str) {
                    enqueue_pending_file(name, bytes);
                }
            }
        }

        // Process queued files (from Intent click or Drag & Drop from Google Drive / Downloads)
        let files_to_open = take_pending_files();
        for (name, bytes) in files_to_open {
            log::info!("Opening document in PhotoCraft: {} ({} bytes)", name, bytes.len());
            match &mut self.state {
                AndroidState::Running(app) => {
                    match app.open_file(&name, &bytes) {
                        Ok(warnings) => {
                            log::info!("Successfully opened {name}. Warnings: {warnings:?}");
                        }
                        Err(e) => {
                            log::error!("Failed to open {name}: {e}");
                        }
                    }
                }
                AndroidState::Onboarding(_) => {
                    let session = self.session.take().unwrap_or_default();
                    let services = self.services.take().unwrap_or_default();
                    let mut app = PhotocraftApp::new(session, services);
                    app.set_theme(ctx, ThemeKind::Pro);
                    if let Some(rs) = self.wgpu_state.clone() {
                        app.set_wgpu(rs);
                    }
                    match app.open_file(&name, &bytes) {
                        Ok(warnings) => {
                            log::info!("Successfully opened {name} from onboarding. Warnings: {warnings:?}");
                        }
                        Err(e) => {
                            log::error!("Failed to open {name}: {e}");
                        }
                    }
                    self.state = AndroidState::Running(Box::new(app));
                }
            }
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

        // Bug 6: Inset spacer to prevent Googlebook OS system window caption bar from covering PhotoCraft
        let total_top_offset = (self.caption_top_inset_points + self.manual_top_offset).max(0.0);
        if total_top_offset > 0.0 {
            egui::Panel::top("googlebook_caption_spacer")
                .exact_size(total_top_offset)
                .frame(egui::Frame::NONE)
                .show(ui, |_ui| {
                    // Transparent spacer ensuring options bar starts cleanly below window caption
                });
        }

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
                        ui.label(egui::RichText::new("PhotoCraft").small().strong());
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
                        ui.label(egui::RichText::new("Barra:").small());
                        if ui.button("➖").clicked() {
                            self.manual_top_offset = (self.manual_top_offset - 4.0).max(-40.0);
                        }
                        ui.label(egui::RichText::new(format!("{:.0}pt", total_top_offset)).small());
                        if ui.button("➕").clicked() {
                            self.manual_top_offset = (self.manual_top_offset + 4.0).min(100.0);
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
        } else if switch_to_onboarding {
            let mut data = OnboardingData::default();
            data.zoom_factor = self.zoom_factor;
            data.permissions_requested = storage_granted;
            self.state = AndroidState::Onboarding(data);
        }

        // Bug 1: Update mouse hover pointer icon right after UI evaluation
        let current_cursor = ctx.output(|o| o.cursor_icon);
        if current_cursor != self.last_cursor_icon {
            set_android_pointer_icon(current_cursor);
            self.last_cursor_icon = current_cursor;
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

    log::info!("PhotoCraft Android & Googlebook OS v0.1.5 starting up");

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
