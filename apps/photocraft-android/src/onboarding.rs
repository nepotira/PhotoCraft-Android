//! Onboarding wizard for PhotoCraft Android (Nintendo Switch emulator-style flow).
//!
//! Port maintained by Nepo (@nepotira).

use egui::{Color32, CornerRadius, Margin, RichText, Stroke, Ui, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OnboardingStep {
    Welcome,
    Permissions,
    DisplayAndDex,
}

pub struct OnboardingData {
    pub step: OnboardingStep,
    pub storage_granted: bool,
    pub notifications_granted: bool,
    pub dex_mode: bool,
    pub ui_scale: f32,
    pub finished: bool,
}

impl Default for OnboardingData {
    fn default() -> Self {
        Self {
            step: OnboardingStep::Welcome,
            storage_granted: false,
            notifications_granted: false,
            dex_mode: true,
            ui_scale: 0.85,
            finished: false,
        }
    }
}

pub fn render_wizard(ui: &mut egui::Ui, data: &mut OnboardingData) {
    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // Main card container
                egui::Frame::NONE
                    .fill(Color32::from_rgb(26, 28, 34))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(48, 54, 65)))
                    .corner_radius(CornerRadius::same(14))
                    .inner_margin(Margin::same(24))
                    .show(ui, |ui| {
                        ui.set_max_width(520.0);

                        // Header badges / indicators
                        ui.horizontal(|ui| {
                            render_step_pill(ui, "1. Boas-Vindas", data.step == OnboardingStep::Welcome);
                            ui.label("➔");
                            render_step_pill(ui, "2. Permissões", data.step == OnboardingStep::Permissions);
                            ui.label("➔");
                            render_step_pill(ui, "3. Tela & DeX", data.step == OnboardingStep::DisplayAndDex);
                        });

                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(16.0);

                        match data.step {
                            OnboardingStep::Welcome => render_welcome(ui, data),
                            OnboardingStep::Permissions => render_permissions(ui, data),
                            OnboardingStep::DisplayAndDex => render_display_dex(ui, data),
                        }
                    });

                ui.add_space(24.0);
            });
        });
    });
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
    ui.heading(RichText::new("🎨 Bem-vindo ao PhotoCraft Android").size(22.0).strong());
    ui.label(RichText::new("Versão v0.0.1-android (Baseado no PhotoCraft v0.8.2)").color(Color32::from_rgb(156, 163, 175)).size(13.0));

    ui.add_space(12.0);
    ui.label(
        "O PhotoCraft é uma suíte criativa completa de edição de imagens escrita em Rust. \
        Esta versão foi especialmente adaptada por Nepo para funcionar perfeitamente em telas de toque \
        e estações desktop Android (como Samsung DeX e AluminiumOS)."
    );

    ui.add_space(14.0);
    egui::Frame::NONE
        .fill(Color32::from_rgb(34, 38, 46))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.label(RichText::new("✨ Destaques desta edição:").strong());
            ui.label("• 🖼️ Suporte nativo a camadas, máscaras e arquivos PSD reais");
            ui.label("• 📱 Escala adaptável para telas menores de celular");
            ui.label("• 🖥️ Janelas redimensionáveis no modo Samsung DeX");
            ui.label("• ⚡ Aceleração gráfica por GPU com WGPU/Vulkan");
        });

    ui.add_space(20.0);
    if ui.add(egui::Button::new(RichText::new("Próximo: Permissões ➔").size(15.0).strong()).min_size(Vec2::new(200.0, 36.0))).clicked() {
        data.step = OnboardingStep::Permissions;
    }
}

fn render_permissions(ui: &mut Ui, data: &mut OnboardingData) {
    ui.heading(RichText::new("📁 Permissões de Armazenamento").size(20.0).strong());
    ui.add_space(8.0);
    ui.label(
        "Para que você possa abrir fotos, arquivos PSD, salvar seus trabalhos e carregar pincéis (.abr) \
        de qualquer pasta do celular ou cartão SD, precisamos do seu consentimento de armazenamento."
    );

    ui.add_space(16.0);

    let storage_btn_text = if data.storage_granted {
        "✓ Acesso ao Armazenamento Habilitado"
    } else {
        "📂 Conceder Acesso ao Armazenamento"
    };

    let btn_color = if data.storage_granted {
        Color32::from_rgb(34, 197, 94)
    } else {
        Color32::from_rgb(59, 130, 246)
    };

    if ui.add(egui::Button::new(RichText::new(storage_btn_text).color(Color32::WHITE).strong()).fill(btn_color).min_size(Vec2::new(ui.available_width(), 40.0))).clicked() {
        data.storage_granted = true;
    }

    ui.add_space(12.0);

    let notif_text = if data.notifications_granted {
        "✓ Notificações de Exportação Habilitadas"
    } else {
        "🔔 Habilitar Notificações (Ao concluir exportações)"
    };

    if ui.add(egui::Button::new(RichText::new(notif_text).size(13.0)).min_size(Vec2::new(ui.available_width(), 34.0))).clicked() {
        data.notifications_granted = true;
    }

    ui.add_space(20.0);
    ui.horizontal(|ui| {
        if ui.button("⬅ Voltar").clicked() {
            data.step = OnboardingStep::Welcome;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(egui::Button::new(RichText::new("Próximo: Modo de Tela ➔").size(14.0).strong()).min_size(Vec2::new(180.0, 34.0))).clicked() {
                data.step = OnboardingStep::DisplayAndDex;
            }
        });
    });
}

fn render_display_dex(ui: &mut Ui, data: &mut OnboardingData) {
    ui.heading(RichText::new("📱 Ajuste de Tela e Samsung DeX").size(20.0).strong());
    ui.add_space(8.0);
    ui.label(
        "Como celulares possuem telas compactas, você pode calibrar a escala da interface \
        para que nenhuma ferramenta fique cortada ou escondida nas bordas."
    );

    ui.add_space(16.0);

    ui.horizontal(|ui| {
        ui.label("Escala da Interface:");
        ui.add(egui::Slider::new(&mut data.ui_scale, 0.6..=1.4).text("zoom"));
    });

    ui.label(
        RichText::new(format!("Escala selecionada: {:.0}% (Recomendado: 80-90% em smartphones, 100-110% em DeX/Tablets)", data.ui_scale * 100.0))
            .size(11.0)
            .color(Color32::from_rgb(156, 163, 175))
    );

    ui.add_space(12.0);
    ui.checkbox(&mut data.dex_mode, "Ativar otimizações para Samsung DeX / AluminiumOS");
    ui.label(RichText::new("Habilita redimensionamento livre de janela e atalhos físicos de teclado/mouse.").size(11.0).color(Color32::from_rgb(140, 145, 155)));

    ui.add_space(24.0);
    ui.horizontal(|ui| {
        if ui.button("⬅ Voltar").clicked() {
            data.step = OnboardingStep::Permissions;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let start_btn = egui::Button::new(
                RichText::new("🚀 Concluir e Iniciar o PhotoCraft!")
                    .color(Color32::WHITE)
                    .size(15.0)
                    .strong()
            )
            .fill(Color32::from_rgb(34, 197, 94))
            .min_size(Vec2::new(240.0, 42.0));

            if ui.add(start_btn).clicked() {
                data.finished = true;
            }
        });
    });
}
