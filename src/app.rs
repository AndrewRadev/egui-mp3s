use std::path::PathBuf;
use std::sync::mpsc::{Sender, Receiver};
use std::ops::Deref;
use std::time::Instant;

use egui::{self, TextureId};
use image::GenericImageView;
use log::debug;
use id3::TagLike;

use crate::music::{MusicFilter, MusicList};
use crate::player::Player;

pub enum WorkerEvent {
    UpdateFilter(MusicFilter)
}

pub enum UiEvent {
    UpdateList(MusicList),
    SetLoading(bool),
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct Mp3sApp {
    filter: MusicFilter,
    list: MusicList,
    player: Player,
    loading: bool,

    selected_path: Option<PathBuf>,
    selected_texture: Option<TextureId>,

    #[cfg_attr(feature = "persistence", serde(skip))]
    worker_sender: Option<Sender<WorkerEvent>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    ui_receiver: Option<Receiver<UiEvent>>,
}

impl Default for Mp3sApp {
    fn default() -> Self {
        let root_dir = ::dirs::audio_dir().
            or_else(|| ::dirs::home_dir()).
            unwrap_or_else(|| ::std::env::current_dir().unwrap_or_default());
        let root_dir = format!("{}", root_dir.display());

        let filter = MusicFilter { root_dir, query: String::new() };
        let list = MusicList { songs: Vec::new() };
        let player = Player::default();

        Self {
            filter, list, player,
            loading: false,
            selected_path: None::<PathBuf>,
            selected_texture: None::<TextureId>,
            worker_sender: None, ui_receiver: None,
        }
    }
}

impl Mp3sApp {
    pub fn new(sender: Sender<WorkerEvent>, receiver: Receiver<UiEvent>) -> Self {
        Self {
            worker_sender: Some(sender),
            ui_receiver: Some(receiver),
            .. Self::default()
        }
    }

    pub fn refresh_filter(&self) {
        self.worker_sender.as_ref().
            map(|s| s.send(WorkerEvent::UpdateFilter(self.filter.clone())));
    }
}

impl eframe::App for Mp3sApp {
    // fn setup(
    //     &mut self,
    //     _ctx: &egui::Context,
    //     _frame: &eframe::Frame,
    //     _storage: Option<&dyn epi::Storage>,
    // ) {
    //     #[cfg(feature = "persistence")]
    //     if let Some(storage) = _storage {
    //         let worker_sender = self.worker_sender.take();
    //         let ui_receiver = self.ui_receiver.take();
    //
    //         *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
    //
    //         self.worker_sender = worker_sender;
    //         self.ui_receiver = ui_receiver;
    //     }
    //
    //     self.refresh_filter();
    // }

    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let start_time = Instant::now();

        if let Some(receiver) = &self.ui_receiver {
            while let Ok(ui_event) = receiver.try_recv() {
                match ui_event {
                    UiEvent::UpdateList(new_music_list) => self.list = new_music_list,
                    UiEvent::SetLoading(loading) => self.loading = loading,
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }

                    if ui.button("Refresh").clicked() {
                        self.refresh_filter();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Music directory: ");
                if ui.text_edit_singleline(&mut self.filter.root_dir).changed() {
                    self.refresh_filter();
                }
            });

            if ui.button("Refresh").clicked() {
                self.refresh_filter();
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

                    // Note: Slow, should happen in the background maybe
                    if self.selected_texture.is_none() {
                        if let Some(image_tag) = tag.pictures().next() {
                            if let Ok(image) = ::image::load_from_memory(&image_tag.data) {
                                let image = image.thumbnail(300, 300);
                                let dimensions = image.dimensions();

                                let egui_image = egui::ColorImage::from_rgba_unmultiplied(
                                    [dimensions.0 as usize, dimensions.1 as usize],
                                    image.to_rgba8().deref(),
                                );

                                // TODO: This needs to be run only once:
                                // HashMap<PathBuf, TextureHandle>
                                let label = format!("{}", path.display());
                                let handle = ctx.load_texture(label, egui_image, egui::TextureOptions::default());

                                self.selected_texture = Some(handle.id());
                            }
                        }
                    }
                }
            }

            if self.player.is_playing() {
                if ui.button("Pause").clicked() {
                    self.player.pause();
                }
            } else if let Some(path) = &self.selected_path {
                if ui.button("Play").clicked() {
                    self.player.play(&PathBuf::from(&self.filter.root_dir).join(path));
                }
            }

            if let Some(texture_id) = self.selected_texture {
                ui.image(texture_id, egui::Vec2 { x: 200.0, y: 200.0 });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.loading {
                ui.heading("List of mp3s 🔁");
            } else {
                ui.heading("List of mp3s");
            }

            ui.label("Filter: ");
            if ui.text_edit_singleline(&mut self.filter.query).changed() {
                self.refresh_filter();
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for filename in &self.list.songs {
                    let selected = Some(filename) == self.selected_path.as_ref();

                    if ui.selectable_label(selected, format!("{}", filename.display())).clicked() {
                        self.selected_texture.take();

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

        debug!("Update took: {}ms", start_time.elapsed().as_millis());
    }
}
