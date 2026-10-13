//! Onboarding wizard for PhotoCraft Android (Nintendo Switch emulator-style flow).
//!
//! Port maintained by Nepo (@nepotira).
//! Version: 0.1.1-android

use egui::{Color32, CornerRadius, Margin, RichText, Stroke, Ui, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OnboardingStep {
    Welcome,
    Permissions,
    DisplayAndDex,
}

pub enum OnboardingAction {
    RequestSystemPermissions,
    OpenStorageSettings,
}

pub struct OnboardingData {
    pub step: OnboardingStep,
    pub permissions_requested: bool,
    pub notifications_requested: bool,
    pub dex_mode: bool,
    pub zoom_factor: f32,
    pub finished: bool,
}

impl Default for OnboardingData {
    fn default() -> Self {
        Self {
            step: OnboardingStep::Welcome,
            permissions_requested: false,
            notifications_requested: false,
            dex_mode: true,
            zoom_factor: 1.0,
            finished: false,
        }
    }
}

pub fn render_wizard(ui: &mut egui::Ui, data: &mut OnboardingData) -> Option<OnboardingAction> {
    let mut triggered_action = None;

    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let available_w = ui.available_width();
            let card_w = (available_w - 24.0).clamp(280.0, 520.0);

            ui.vertical_centered(|ui| {
                ui.add_space(16.0);

                // Main responsive card container
                egui::Frame::NONE
                    .fill(Color32::from_rgb(26, 28, 34))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(48, 54, 65)))
                    .corner_radius(CornerRadius::same(14))
                    .inner_margin(Margin::symmetric(16, 20))
                    .show(ui, |ui| {
                        ui.set_max_width(card_w);

                        // Header badges / indicators
                        ui.horizontal_wrapped(|ui| {
                            render_step_pill(ui, "1. Boas-Vindas", data.step == OnboardingStep::Welcome);
                            ui.label(RichText::new("➔").color(Color32::GRAY));
                            render_step_pill(ui, "2. Permissões", data.step == OnboardingStep::Permissions);
                            ui.label(RichText::new("➔").color(Color32::GRAY));
                            render_step_pill(ui, "3. Escala & DeX", data.step == OnboardingStep::DisplayAndDex);
                        });

                        ui.add_space(14.0);
                        ui.separator();
                        ui.add_space(14.0);

                        match data.step {
                            OnboardingStep::Welcome => render_welcome(ui, data),
                            OnboardingStep::Permissions => {
                                if let Some(action) = render_permissions(ui, data) {
                                    triggered_action = Some(action);
                                }
                            }
                            OnboardingStep::DisplayAndDex => render_display_dex(ui, data),
                        }
                    });

                ui.add_space(20.0);
            });
        });
    });

    triggered_action
}

fn render_step_pill(ui: &mut Ui, label: &str, active: bool) {
    let (bg, text_col) = if active {
        (Color32::from_rgb(59, 130, 246), Color32::WHITE)
    } else {
        (Color32::from_rgb(38, 42, 50), Color32::from_rgb(150, 158, 175))
    };

    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(label).color(text_col).size(12.0).strong());
        });
}

fn render_welcome(ui: &mut Ui, data: &mut OnboardingData) {
    ui.heading(RichText::new("🎨 PhotoCraft Android").size(20.0).strong());
    ui.label(
        RichText::new("Versão v0.1.1-android (Base upstream v0.8.2)")
            .color(Color32::from_rgb(156, 163, 175))
            .size(12.0),
    );

    ui.add_space(12.0);
    ui.label(
        "Suíte criativa profissional de edição de imagens em Rust. \
        Port oficial para Android mantido por Nepo com suporte nativo a touch e Samsung DeX.",
    );

    ui.add_space(12.0);
    egui::Frame::NONE
        .fill(Color32::from_rgb(34, 38, 46))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.label(RichText::new("✨ Destaques desta versão:").strong());
            ui.label("• 🖼️ Suporte completo a camadas, máscaras e arquivos PSD reais");
            ui.label("• 📱 Tela cheia imersiva sem sobreposição de barras do sistema");
            ui.label("• 🖥️ Janelas livres e redimensionáveis no Samsung DeX");
            ui.label("• ⚡ Aceleração gráfica nativa por GPU (WGPU / Vulkan)");
        });

    ui.add_space(18.0);
    let next_btn = egui::Button::new(RichText::new("Próximo: Permissões ➔").size(14.0).strong())
        .min_size(Vec2::new(ui.available_width(), 44.0))
        .fill(Color32::from_rgb(59, 130, 246));

    if ui.add(next_btn).clicked() {
        data.step = OnboardingStep::Permissions;
    }
}

fn render_permissions(ui: &mut Ui, data: &mut OnboardingData) -> Option<OnboardingAction> {
    let mut action = None;

    ui.heading(RichText::new("📁 Permissões de Armazenamento").size(19.0).strong());
    ui.add_space(6.0);
    ui.label(
        "Para abrir suas fotos, editar PSDs e exportar trabalhos, o aplicativo precisa da sua autorização no Android.",
    );

    ui.add_space(14.0);

    // Primary system permission request button (calls JNI requestPermissions)
    let perm_btn_text = if data.permissions_requested {
        "✓ Solicitação do Sistema Enviada"
    } else {
        "🛡️ Solicitar Permissões do Android"
    };

    let perm_btn_color = if data.permissions_requested {
        Color32::from_rgb(16, 185, 129)
    } else {
        Color32::from_rgb(59, 130, 246)
    };

    let req_btn = egui::Button::new(
        RichText::new(perm_btn_text)
            .color(Color32::WHITE)
            .size(14.0)
            .strong(),
    )
    .fill(perm_btn_color)
    .min_size(Vec2::new(ui.available_width(), 44.0));

    if ui.add(req_btn).clicked() {
        data.permissions_requested = true;
        action = Some(OnboardingAction::RequestSystemPermissions);
    }

    ui.add_space(6.0);
    ui.label(
        RichText::new("Abre o diálogo nativo do Android para fotos, mídia e notificações.")
            .color(Color32::from_rgb(156, 163, 175))
            .size(11.0),
    );

    ui.add_space(10.0);

    // Secondary button: Android 11+ All Files Access
    let settings_btn = egui::Button::new(
        RichText::new("📂 Acesso a Todos os Arquivos (Configurações)")
            .size(13.0)
            .color(Color32::from_rgb(220, 225, 235)),
    )
    .fill(Color32::from_rgb(45, 52, 64))
    .min_size(Vec2::new(ui.available_width(), 40.0));

    if ui.add(settings_btn).clicked() {
        action = Some(OnboardingAction::OpenStorageSettings);
    }

    ui.add_space(4.0);
    ui.label(
        RichText::new("Opcional: abre a tela de gerenciamento de arquivos para acessar cartões SD e pastas completas.")
            .color(Color32::from_rgb(156, 163, 175))
            .size(11.0),
    );

    ui.add_space(18.0);
    ui.horizontal(|ui| {
        if ui
            .add(egui::Button::new("⬅ Voltar").min_size(Vec2::new(80.0, 38.0)))
            .clicked()
        {
            data.step = OnboardingStep::Welcome;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let next_btn = egui::Button::new(
                RichText::new("Próximo: Escala ➔")
                    .size(14.0)
                    .strong(),
            )
            .fill(Color32::from_rgb(59, 130, 246))
            .min_size(Vec2::new(140.0, 38.0));

            if ui.add(next_btn).clicked() {
                data.step = OnboardingStep::DisplayAndDex;
            }
        });
    });

    action
}

fn render_display_dex(ui: &mut Ui, data: &mut OnboardingData) {
    ui.heading(RichText::new("📱 Escala e Samsung DeX").size(19.0).strong());
    ui.add_space(6.0);
    ui.label(
        "Ajuste a proporção dos menus e botões para que caibam confortavelmente na tela do seu dispositivo.",
    );

    ui.add_space(14.0);

    ui.horizontal(|ui| {
        ui.label("Zoom da Interface:");
        ui.add(egui::Slider::new(&mut data.zoom_factor, 0.7..=1.4).text(""));
    });

    ui.label(
        RichText::new(format!(
            "Zoom atual: {:.0}% (Recomendado: 85-100% para celular, 100-115% para tablets/DeX)",
            data.zoom_factor * 100.0
        ))
        .size(11.0)
        .color(Color32::from_rgb(156, 163, 175)),
    );

    ui.add_space(12.0);
    ui.checkbox(&mut data.dex_mode, "Otimizações de Área de Trabalho (Samsung DeX)");
    ui.label(
        RichText::new("Habilita redimensionamento livre de janelas e adaptações de ponteiro do mouse.")
            .size(11.0)
            .color(Color32::from_rgb(140, 145, 155)),
    );

    ui.add_space(20.0);
    ui.horizontal(|ui| {
        if ui
            .add(egui::Button::new("⬅ Voltar").min_size(Vec2::new(80.0, 42.0)))
            .clicked()
        {
            data.step = OnboardingStep::Permissions;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let start_btn = egui::Button::new(
                RichText::new("🚀 Abrir o PhotoCraft!")
                    .color(Color32::WHITE)
                    .size(15.0)
                    .strong(),
            )
            .fill(Color32::from_rgb(16, 185, 129))
            .min_size(Vec2::new(190.0, 42.0));

            if ui.add(start_btn).clicked() {
                data.finished = true;
            }
        });
    });
}
