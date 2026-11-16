use crate::app::ArcticApp;
use crate::art::rico_logo_texture;
use crate::theme;
use crate::widgets;
use arctic_core::{APP_NAME, APP_VERSION};
use eframe::egui::{self, RichText, Sense, vec2};

pub(crate) const DISCORD: &str = "https://discord.arcticlauncher.com";

impl ArcticApp {
    pub(crate) fn about_tab(&mut self, ui: &mut egui::Ui) {
        let p = self.palette();
        widgets::page_header(ui, p, crate::i18n::current("About"), "");
        theme::card(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(92.0, 92.0), Sense::hover());
                let texture = rico_logo_texture(ui.ctx());
                let uv = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
                ui.painter().image(texture.id(), rect, uv, egui::Color32::WHITE);
                ui.vertical(|ui| {
                    ui.label(RichText::new(APP_NAME).size(28.0).strong().color(p.text));
                    ui.label(RichText::new(format!("{} {APP_VERSION}", crate::i18n::current("Version"))).color(p.muted));
                });
            });
            ui.add_space(14.0);
            ui.label(
                RichText::new(
                    crate::i18n::current("RICO is an open-source Minecraft launcher for managing game versions, instances, mods, accounts, and friends. It includes the RICO Client and tools for organizing your Minecraft setup.")
                )
                .size(16.0)
                .color(p.text),
            );
        });
    }
}
