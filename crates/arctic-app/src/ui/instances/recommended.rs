//! Curated, one-click performance and visual mods from their official Modrinth projects.

use std::collections::HashSet;

use arctic_core::instances::Instance;
use eframe::egui::{self, RichText};

use super::mods_page::mod_icon;
use crate::app::ArcticApp;
use crate::art::icons::Icon;
use crate::theme;
use crate::widgets;

struct FeaturedMod {
    project_id: &'static str,
    title: &'static str,
    category: &'static str,
    description: &'static str,
    caveat: &'static str,
}

// Project IDs identify the official Modrinth listings. The installer resolves the exact file
// against the selected instance's Minecraft version and loader before downloading.
const FEATURED: &[FeaturedMod] = &[
    FeaturedMod {
        project_id: "AANobbMI",
        title: "Sodium",
        category: "FPS & rendering",
        description: "Optimizes Minecraft's rendering engine for smoother gameplay and higher FPS.",
        caveat: "Client-side only. Modrinth selects a build for the instance's loader and Minecraft version.",
    },
    FeaturedMod {
        project_id: "gvQqBUqZ",
        title: "Lithium",
        category: "Performance",
        description: "Optimizes game logic to reduce CPU load without changing the vanilla experience.",
        caveat: "Works on client or server; Modrinth selects a build for the instance's loader and version.",
    },
    FeaturedMod {
        project_id: "uXXizFIs",
        title: "FerriteCore",
        category: "Memory",
        description: "Reduces memory use by optimizing how Minecraft stores duplicated data.",
        caveat: "Works on client or server. Avoid combining with Hydrogen on Minecraft 1.18 and newer.",
    },
    FeaturedMod {
        project_id: "5ZwdcRci",
        title: "ImmediatelyFast",
        category: "FPS & rendering",
        description: "Optimizes immediate-mode rendering for interfaces, entities, particles and more.",
        caveat: "Client-side only; does not guarantee higher FPS if rendering is not the bottleneck.",
    },
    FeaturedMod {
        project_id: "NNAgCjsB",
        title: "Entity Culling",
        category: "FPS & rendering",
        description: "Skips rendering entities and block entities that are hidden from view.",
        caveat: "Client-side only. A few animated or oversized models may need a whitelist.",
    },
    FeaturedMod {
        project_id: "nmDcB62a",
        title: "ModernFix",
        category: "Performance & memory",
        description: "A collection of fixes and optimizations for memory use, loading and performance.",
        caveat: "Optional on client or server. Features and loader support vary by game version.",
    },
    FeaturedMod {
        project_id: "YL57xq9U",
        title: "Iris Shaders",
        category: "Graphics",
        description: "Loads shader packs to customize Minecraft's lighting and atmosphere.",
        caveat: "Shader packs may increase GPU load. Modrinth resolves the required Sodium dependency when available.",
    },
];

impl ArcticApp {
    pub(super) fn recommended_mods_page(&mut self, ui: &mut egui::Ui, instance: &Instance) {
        let p = self.palette();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("RICO Library")
                        .size(19.0)
                        .strong()
                        .color(p.text),
                );
                ui.label(
                    RichText::new(crate::i18n::current(
                        "Curated performance and graphics mods. Files are installed from Modrinth.",
                    ))
                    .color(p.muted),
                );
            });
        });
        ui.add_space(10.0);

        let installed: HashSet<String> = self
            .inst
            .mod_scans
            .get(&instance.id)
            .and_then(|scan| scan.data.as_ref())
            .map(|data| {
                data.files
                    .iter()
                    .filter_map(|file| {
                        file.tracked
                            .as_ref()
                            .map(|tracked| tracked.project_id.clone())
                    })
                    .collect()
            })
            .unwrap_or_default();

        for item in FEATURED {
            theme::card(p).show(ui, |ui| {
                ui.set_width(ui.available_width());
                crate::widgets::row(ui, |ui| {
                    ui.scope(|ui| mod_icon(ui, self, None));
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(item.title).strong().size(16.0).color(p.text));
                            ui.label(RichText::new(item.category).small().color(p.accent));
                        });
                        ui.label(RichText::new(item.description).color(p.muted));
                        ui.label(RichText::new(item.caveat).small().color(p.muted));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let busy = self.inst.installing.contains(item.project_id);
                        if installed.contains(item.project_id) {
                            ui.label(
                                RichText::new(crate::i18n::current("Installed")).color(p.accent),
                            );
                        } else if busy {
                            ui.spinner();
                        } else if widgets::button(
                            ui,
                            p,
                            Some(Icon::Plus),
                            crate::i18n::current("Install"),
                            true,
                        )
                        .clicked()
                        {
                            self.inst.installing.insert(item.project_id.to_owned());
                            self.tasks.mod_install(
                                instance.clone(),
                                item.project_id.to_owned(),
                                None,
                            );
                        }
                    });
                });
            });
            ui.add_space(6.0);
        }

        if !self.inst.installing.is_empty() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    RichText::new(crate::i18n::current(
                        "Downloading compatible builds from Modrinth…",
                    ))
                    .color(p.muted),
                );
            });
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(200));
        }
    }
}
