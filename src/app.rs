use std::path::PathBuf;
use std::sync::mpsc;

use eframe::{egui, epi};

use crate::music::{MusicFilter, MusicList};

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct Mp3sApp {
    filter: MusicFilter,
    list: MusicList,

    selected_path: Option<PathBuf>,

    #[cfg_attr(feature = "persistence", serde(skip))]
    filter_sender: Option<mpsc::Sender<MusicFilter>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    list_receiver: Option<mpsc::Receiver<MusicList>>,
}

impl Default for Mp3sApp {
    fn default() -> Self {
        let root_dir = ::dirs::audio_dir().
            or_else(|| ::dirs::home_dir()).
            unwrap_or_else(|| ::std::env::current_dir().unwrap());
        let root_dir = format!("{}", root_dir.display());

        let filter = MusicFilter { root_dir, filter: String::new() };
        let list = MusicList { loading: true, songs: Vec::new() };

        Self {
            filter, list,
            selected_path: None::<PathBuf>,
            filter_sender: None, list_receiver: None,
        }
    }
}

impl Mp3sApp {
    fn new(sender: mpsc::Sender<MusicFilter>, receiver: mpsc::Receiver<MusicList>) -> Self {
        Self {
            filter_sender: Some(sender),
            list_receiver: Some(receiver),
            .. Self::default()
        }
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
            let filter_sender = self.filter_sender.take();
            let list_receiver = self.list_receiver.take();

            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();

            self.filter_sender = filter_sender;
            self.list_receiver = list_receiver;
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
                ui.text_edit_singleline(&mut self.filter.root_dir);
            });

            if ui.button("Refresh").clicked() {
                println!("TODO: Refresh");
            }

            if let Some(path) = &self.selected_path {
                if let Ok(tag) = ::id3::Tag::read_from_path(PathBuf::from(&self.filter.root_dir).join(path)) {
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
            ui.text_edit_singleline(&mut self.filter.filter);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for entry in std::fs::read_dir(&self.filter.root_dir).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();

                    if path.extension() != Some("mp3".as_ref()) {
                        continue;
                    }

                    let filename = path.strip_prefix(&self.filter.root_dir).unwrap();
                    if self.filter.filter.trim() != "" &&
                        !filename.to_string_lossy().to_lowercase().contains(&self.filter.filter.to_lowercase()) {
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
