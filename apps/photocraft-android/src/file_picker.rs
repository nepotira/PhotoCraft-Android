//! Android integrated file picker modal for PhotoCraft.
//!
//! Provides direct file browsing for /sdcard/Download, /sdcard/Pictures, etc.
//! Answers Photocraft's FileDialogReply cleanly without freezing the event loop.

use egui::{Color32, RichText, ScrollArea, Vec2};
use photocraft_ui_egui::{FileDialogAnswer, FileDialogReply, FileDialogRequest};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct PendingFilePicker {
    pub request: FileDialogRequest,
    pub reply: FileDialogReply,
    pub current_dir: PathBuf,
    pub selected_file: Option<PathBuf>,
    pub save_name: String,
    pub show_all_files: bool,
    pub error_msg: Option<String>,
}

#[derive(Clone, Default)]
pub struct FilePickerState {
    pending: Arc<Mutex<Option<PendingFilePicker>>>,
}

impl FilePickerState {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(None)),
        }
    }

    pub fn open_dialog(&self, request: FileDialogRequest, reply: FileDialogReply) {
        let initial_dir = match &request {
            FileDialogRequest::Open { initial_dir, .. } => initial_dir
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(default_storage_dir),
            FileDialogRequest::Folder { initial_dir } => initial_dir
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(default_storage_dir),
            FileDialogRequest::Save { suggested } => Path::new(suggested)
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(default_storage_dir),
        };

        let save_name = match &request {
            FileDialogRequest::Save { suggested } => Path::new(suggested)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "documento.png".to_string()),
            _ => String::new(),
        };

        let current_dir = if initial_dir.exists() {
            initial_dir
        } else {
            default_storage_dir()
        };

        let mut lock = self.pending.lock().unwrap();
        *lock = Some(PendingFilePicker {
            request,
            reply,
            current_dir,
            selected_file: None,
            save_name,
            show_all_files: false,
            error_msg: None,
        });
    }

    pub fn ui(&self, ctx: &egui::Context) {
        let mut should_close = false;
        let mut final_answer: Option<Option<FileDialogAnswer>> = None;

        let mut lock = self.pending.lock().unwrap();
        if let Some(picker) = lock.as_mut() {
            let title = match picker.request {
                FileDialogRequest::Open { multiple, .. } => {
                    if multiple { "Abrir Arquivos (Android)" } else { "Abrir Arquivo (Android)" }
                }
                FileDialogRequest::Folder { .. } => "Selecionar Pasta (Android)",
                FileDialogRequest::Save { .. } => "Exportar / Salvar Como (Android)",
            };

            let screen_rect = ctx.content_rect();
            let window_w = (screen_rect.width() - 32.0).clamp(300.0, 600.0);
            let window_h = (screen_rect.height() - 48.0).clamp(320.0, 700.0);

            egui::Window::new(RichText::new(title).strong())
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .fixed_size(Vec2::new(window_w, window_h))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.set_width(window_w - 24.0);

                    let root_storage = if Path::new("/storage/emulated/0").exists() {
                        "/storage/emulated/0"
                    } else if Path::new("/storage/emulated/10").exists() {
                        "/storage/emulated/10"
                    } else {
                        "/sdcard"
                    };

                    // Shortcuts row
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Locais:").small().color(Color32::GRAY));
                        if ui.button("📥 Downloads").clicked() {
                            let p = PathBuf::from(format!("{root_storage}/Download"));
                            picker.current_dir = if p.exists() { p } else { PathBuf::from("/sdcard/Download") };
                            picker.selected_file = None;
                        }
                        if ui.button("🖼️ Imagens").clicked() {
                            let p = PathBuf::from(format!("{root_storage}/Pictures"));
                            picker.current_dir = if p.exists() { p } else { PathBuf::from("/sdcard/Pictures") };
                            picker.selected_file = None;
                        }
                        if ui.button("📷 Câmera").clicked() {
                            let p = PathBuf::from(format!("{root_storage}/DCIM"));
                            picker.current_dir = if p.exists() { p } else { PathBuf::from("/sdcard/DCIM") };
                            picker.selected_file = None;
                        }
                        if ui.button("📁 Armazenamento").clicked() {
                            picker.current_dir = PathBuf::from(root_storage);
                            picker.selected_file = None;
                        }
                    });

                    ui.separator();

                    // Current Directory & Up button
                    ui.horizontal(|ui| {
                        if ui.button("⬆ Subir Pasta").clicked() {
                            if let Some(parent) = picker.current_dir.parent() {
                                if parent.starts_with("/sdcard") || parent.starts_with("/storage") {
                                    picker.current_dir = parent.to_path_buf();
                                    picker.selected_file = None;
                                }
                            }
                        }
                        ui.label(
                            RichText::new(format!("Pasta: {}", picker.current_dir.display()))
                                .small()
                                .color(Color32::from_rgb(180, 190, 205)),
                        );
                    });

                    let allowed_extensions: Option<Vec<String>> = match &picker.request {
                        FileDialogRequest::Open { extensions: Some(exts), .. } => Some(exts.clone()),
                        _ => None,
                    };

                    if let Some(ref exts) = allowed_extensions {
                        if !exts.is_empty() {
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("Filtro: .{}", exts.join(", ."))).small().color(Color32::from_rgb(140, 200, 255)));
                                ui.checkbox(&mut picker.show_all_files, "Exibir todos os formatos");
                            });
                        }
                    }

                    ui.separator();

                    // List files and directories
                    let mut chosen_entry: Option<PathBuf> = None;
                    let is_save = matches!(picker.request, FileDialogRequest::Save { .. });

                    ScrollArea::vertical()
                        .max_height(window_h - 180.0)
                        .show(ui, |ui| {
                            match fs::read_dir(&picker.current_dir) {
                                Ok(entries) => {
                                    let mut dirs = Vec::new();
                                    let mut files = Vec::new();

                                    for entry in entries.flatten() {
                                        let path = entry.path();
                                        if path.is_dir() {
                                            dirs.push(path);
                                        } else if path.is_file() {
                                            files.push(path);
                                        }
                                    }

                                    dirs.sort();
                                    files.sort();

                                    for d in dirs {
                                        let name = d.file_name().unwrap_or_default().to_string_lossy();
                                        if name.starts_with('.') {
                                            continue;
                                        }
                                        if ui.selectable_label(false, format!("📁 {}/", name)).clicked() {
                                            chosen_entry = Some(d);
                                        }
                                    }

                                    let mut filtered_files: Vec<PathBuf> = Vec::new();
                                    for f in files {
                                        let name = f.file_name().unwrap_or_default().to_string_lossy();
                                        if name.starts_with('.') {
                                            continue;
                                        }
                                        if !picker.show_all_files {
                                            if let Some(ref exts) = allowed_extensions {
                                                if !exts.is_empty() {
                                                    let matches = f.extension()
                                                        .and_then(|e| e.to_str())
                                                        .map(|ext| exts.iter().any(|a| a.eq_ignore_ascii_case(ext)))
                                                        .unwrap_or(false);
                                                    if !matches {
                                                        continue;
                                                    }
                                                }
                                            }
                                        }
                                        filtered_files.push(f);
                                    }

                                    for f in &filtered_files {
                                        let name = f.file_name().unwrap_or_default().to_string_lossy();
                                        let is_selected = picker.selected_file.as_ref() == Some(f);
                                        if ui.selectable_label(is_selected, format!("📄 {}", name)).clicked() {
                                            picker.selected_file = Some(f.clone());
                                            if is_save {
                                                picker.save_name = name.to_string();
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    ui.colored_label(
                                        Color32::from_rgb(239, 68, 68),
                                        format!("Erro ao acessar pasta: {e}\nCertifique-se de que a permissão de armazenamento foi concedida."),
                                    );
                                }
                            }
                        });

                    if let Some(new_dir) = chosen_entry {
                        picker.current_dir = new_dir;
                        picker.selected_file = None;
                    }

                    ui.separator();

                    // Save / Export inputs if saving
                    if is_save {
                        ui.horizontal(|ui| {
                            ui.label("Nome do arquivo:");
                            ui.text_edit_singleline(&mut picker.save_name);
                        });
                        ui.add_space(4.0);
                    }

                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("Cancelar").clicked() {
                            should_close = true;
                            final_answer = Some(None);
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            match &picker.request {
                                FileDialogRequest::Open { multiple, .. } => {
                                    let enabled = picker.selected_file.is_some();
                                    if ui
                                        .add_enabled(enabled, egui::Button::new("Abrir Selecionado"))
                                        .clicked()
                                    {
                                        if let Some(path) = &picker.selected_file {
                                            let path_str = path.to_string_lossy().to_string();
                                            let answer = if *multiple {
                                                FileDialogAnswer::Paths(vec![path_str])
                                            } else {
                                                FileDialogAnswer::Paths(vec![path_str])
                                            };
                                            final_answer = Some(Some(answer));
                                            should_close = true;
                                        }
                                    }
                                }
                                FileDialogRequest::Save { .. } => {
                                    let enabled = !picker.save_name.trim().is_empty();
                                    if ui
                                        .add_enabled(enabled, egui::Button::new("Exportar / Salvar"))
                                        .clicked()
                                    {
                                        let target_path = picker.current_dir.join(&picker.save_name);
                                        let path_str = target_path.to_string_lossy().to_string();
                                        final_answer = Some(Some(FileDialogAnswer::SaveTo(path_str)));
                                        should_close = true;
                                    }
                                }
                                FileDialogRequest::Folder { .. } => {
                                    if ui.button("Escolher Esta Pasta").clicked() {
                                        let path_str = picker.current_dir.to_string_lossy().to_string();
                                        final_answer = Some(Some(FileDialogAnswer::FolderTo(path_str)));
                                        should_close = true;
                                    }
                                }
                            }
                        });
                    });
                });
        }

        if should_close {
            if let Some(picker) = lock.take() {
                if let Some(ans) = final_answer {
                    picker.reply.send(ans);
                } else {
                    picker.reply.send(None);
                }
            }
        }
    }
}

fn default_storage_dir() -> PathBuf {
    let candidates = [
        "/storage/emulated/0/Download",
        "/storage/emulated/10/Download",
        "/sdcard/Download",
        "/storage/emulated/0/Pictures",
        "/storage/emulated/10/Pictures",
        "/sdcard/Pictures",
        "/storage/emulated/0",
        "/storage/emulated/10",
        "/sdcard",
    ];
    for &cand in &candidates {
        let p = PathBuf::from(cand);
        if p.exists() && p.is_dir() {
            return p;
        }
    }
    PathBuf::from("/sdcard")
}
