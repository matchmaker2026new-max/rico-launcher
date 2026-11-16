//! صفحة الإحصائيات: أين يذهب وقت اللعب، وتبقى البيانات على هذا الجهاز.

use arctic_core::playtime::{self, Part, ago, format_duration};
use eframe::egui::{self, CornerRadius, RichText, Sense, vec2};

use crate::app::ArcticApp;
use crate::theme;
use crate::widgets;

const TOP: usize = 10;

impl ArcticApp {
    pub(crate) fn stats_tab(&mut self, ui: &mut egui::Ui) {
        let p = self.palette();
        let language = self.settings.language;
        widgets::page_header(
            ui,
            p,
            crate::i18n::text(language, "Stats"),
            crate::i18n::text(language, "Where your time goes. It stays on this PC."),
        );
        let stats = self.playtime.clone();
        let now = arctic_core::auth::now_secs();

        if stats.total_secs() == 0 && stats.launcher_secs() == 0 {
            theme::card(p).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(crate::i18n::text(language, "Nothing yet")).size(17.0).strong().color(p.text));
                ui.label(
                    RichText::new(crate::i18n::text(language, "Play for a while and this page fills up: time per instance, per server and per singleplayer world, and how long you spent in menus."))
                        .color(p.muted),
                );
            });
            return;
        }

        theme::card(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(crate::i18n::text(language, "Total"))
                    .size(18.0)
                    .strong()
                    .color(p.text),
            );
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                let sessions = stats.sessions();
                let tiles = [
                    ("Game open", format_duration(stats.total_secs())),
                    (
                        "In a world or server",
                        format_duration(stats.in_world_secs()),
                    ),
                    ("In menus", format_duration(stats.menu_secs())),
                    ("Launcher open", format_duration(stats.launcher_secs())),
                    ("Sessions", sessions.to_string()),
                ];
                for (index, (label, value)) in tiles.into_iter().enumerate() {
                    if index > 0 {
                        let (rect, _) = ui.allocate_exact_size(vec2(1.0, 40.0), Sense::hover());
                        ui.painter().line_segment(
                            [rect.center_top(), rect.center_bottom()],
                            egui::Stroke::new(1.0, p.card_stroke),
                        );
                        ui.add_space(10.0);
                    }
                    ui.vertical(|ui| {
                        ui.set_min_width(170.0);
                        ui.label(
                            RichText::new(crate::i18n::text(language, label))
                                .small()
                                .color(p.muted),
                        );
                        ui.label(RichText::new(value).size(22.0).strong().color(p.text));
                    });
                }
            });
        });
        ui.add_space(12.0);

        let instances = stats.instances();
        let longest = instances
            .iter()
            .map(|(_, e)| e.total_secs)
            .max()
            .unwrap_or(1)
            .max(1);
        theme::card(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(crate::i18n::text(language, "Instances"))
                    .size(18.0)
                    .strong()
                    .color(p.text),
            );
            ui.add_space(6.0);
            for (id, entry) in &instances {
                let name = self.stats_instance_name(id);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(name).color(p.text));
                    let session_word = if entry.sessions == 1 {
                        "session"
                    } else {
                        "sessions"
                    };
                    let detail = format!(
                        "{} · {} {} · {} {}",
                        format_duration(entry.total_secs),
                        entry.sessions,
                        crate::i18n::text(language, session_word),
                        crate::i18n::text(language, "last"),
                        ago(entry.last_played, now),
                    );
                    ui.label(RichText::new(detail).small().color(p.muted));
                });
                bar(ui, p, entry.total_secs as f32 / longest as f32);
                ui.add_space(6.0);
            }
        });

        let hide_id = egui::Id::new("stats_hide_names");
        let mut hide = ui.data(|d| d.get_temp::<bool>(hide_id)).unwrap_or(false);
        let servers = stats.servers();
        let worlds = stats.worlds();
        if !servers.is_empty() || !worlds.is_empty() {
            ui.add_space(12.0);
            if ui
                .checkbox(
                    &mut hide,
                    crate::i18n::text(language, "Hide server and world names"),
                )
                .changed()
            {
                ui.data_mut(|d| d.insert_temp(hide_id, hide));
            }
            ui.add_space(6.0);
        }
        top_list(ui, p, language, "Servers", &servers, hide, now, "Server");
        top_list(
            ui,
            p,
            language,
            "Singleplayer worlds",
            &worlds,
            hide,
            now,
            "World",
        );
        let _ = playtime::GAME_LOG;
    }

    fn stats_instance_name(&self, id: &str) -> String {
        if self.instance.id == id {
            return self.instance.name.clone();
        }
        self.custom_instances
            .iter()
            .find(|i| i.id == id)
            .map_or_else(
                || crate::i18n::text(self.settings.language, "Removed instance").to_owned(),
                |i| i.name.clone(),
            )
    }
}

fn top_list(
    ui: &mut egui::Ui,
    p: &crate::theme::Palette,
    language: arctic_core::settings::Language,
    title: &str,
    list: &[(String, Part)],
    hide: bool,
    now: u64,
    anonymous: &str,
) {
    if list.is_empty() {
        return;
    }
    ui.add_space(12.0);
    let longest = list.first().map_or(1, |(_, part)| part.secs).max(1);
    theme::card(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            RichText::new(crate::i18n::text(language, title))
                .size(18.0)
                .strong()
                .color(p.text),
        );
        ui.add_space(6.0);
        for (index, (name, part)) in list.iter().take(TOP).enumerate() {
            let shown = if hide {
                format!("{} {}", crate::i18n::text(language, anonymous), index + 1)
            } else {
                name.clone()
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new(shown).color(p.text));
                ui.label(
                    RichText::new(format!(
                        "{} {} {}",
                        format_duration(part.secs),
                        crate::i18n::text(language, "last"),
                        ago(part.last, now)
                    ))
                    .small()
                    .color(p.muted),
                );
            });
            bar(ui, p, part.secs as f32 / longest as f32);
            ui.add_space(6.0);
        }
        if list.len() > TOP {
            ui.label(
                RichText::new(format!(
                    "{} {}",
                    crate::i18n::text(language, "and"),
                    list.len() - TOP
                ))
                .small()
                .color(p.muted),
            );
        }
    });
}

fn bar(ui: &mut egui::Ui, p: &crate::theme::Palette, share: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 6.0), Sense::hover());
    let radius = CornerRadius::same(3);
    ui.painter().rect_filled(rect, radius, p.surface_hover);
    let mut fill = rect;
    fill.set_width((rect.width() * share.clamp(0.0, 1.0)).max(4.0));
    ui.painter().rect_filled(fill, radius, p.accent);
}
