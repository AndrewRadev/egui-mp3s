use std::path::PathBuf;

use eframe::{egui, epi};

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct Mp3sApp {
    music_root: String,
    selected_path: Option<PathBuf>,
    filter: String,
}

impl Default for Mp3sApp {
    fn default() -> Self {
        let music_root = ::dirs::audio_dir().
            or_else(|| ::dirs::home_dir()).
            unwrap_or_else(|| ::std::env::current_dir().unwrap());
        let music_root = format!("{}", music_root.display());

        Self { music_root, selected_path: None::<PathBuf>, filter: String::new() }
    }
}

impl epi::App for Mp3sApp {
    fn name(&self) -> &str {
        "Basic Mp3 Viewer"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }

                    if ui.button("Refresh").clicked() {
                        println!("TODO: Refresh");
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Music directory: ");
                ui.text_edit_singleline(&mut self.music_root);
            });

            if ui.button("Refresh").clicked() {
                println!("TODO: Refresh");
            }

            if let Some(path) = &self.selected_path {
                if let Ok(tag) = ::id3::Tag::read_from_path(PathBuf::from(&self.music_root).join(path)) {
                    if let Some(artist) = tag.artist() {
                        ui.label(format!("Artist: {}", artist));
                    }
                    if let Some(title) = tag.title() {
                        ui.label(format!("Title: {}", title));
                    }
                    if let Some(album) = tag.album() {
                        ui.label(format!("Album: {}", album));
                    }
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("List of mp3s");

            ui.label("Filter: ");
            ui.text_edit_singleline(&mut self.filter);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for entry in std::fs::read_dir(&self.music_root).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();

                    if path.extension() != Some("mp3".as_ref()) {
                        continue;
                    }

                    let filename = path.strip_prefix(&self.music_root).unwrap();
                    if self.filter.trim() != "" &&
                        !filename.to_string_lossy().to_lowercase().contains(&self.filter.to_lowercase()) {
                        continue;
                    }

                    let selected = Some(filename) == self.selected_path.as_ref().map(|p| p.as_path());

                    if ui.selectable_label(selected, format!("{}", filename.display())).clicked() {
                        if selected {
                            self.selected_path = None;
                        } else {
                            self.selected_path = Some(filename.to_owned());
                        }
                    }
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
        });
    }
}
