use std::path::PathBuf;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::collections::HashMap;
use std::ops::Deref;

use walkdir::WalkDir;
use egui::TextureId;
use image::GenericImageView;

use crate::app::{WorkerEvent, UiEvent};

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
#[derive(Clone, Default)]
pub struct MusicFilter {
    /// Root dir is not a `PathBuf` so we can easily plug it into egui's text input
    pub root_dir: String,
    pub query: String,
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
#[derive(Clone, Default)]
pub struct MusicList {
    pub songs: Vec<PathBuf>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub song_images: HashMap<PathBuf, SongImage>,
}

#[derive(Clone, Default)]
pub struct SongImage {
    pub texture_id: Option<TextureId>,
}

impl MusicList {
    pub fn update(&mut self, filter: &MusicFilter) {
        if !PathBuf::from(&filter.root_dir).is_dir() {
            // Not a real directory, don't change the list
            return;
        }

        self.songs.clear();

        for entry in WalkDir::new(&filter.root_dir) {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("[Warning] Couldn't read dir entry: {}", e);
                    continue;
                },
            };
            let path = entry.path();

            if path.extension() != Some("mp3".as_ref()) {
                continue;
            }

            let filename = match path.strip_prefix(&filter.root_dir) {
                Ok(filename) => filename,
                Err(_) => {
                    eprintln!(
                        "[Warning] Path '{}' is not inside our root dir: '{}'",
                        path.display(),
                        filter.root_dir,
                    );
                    continue;
                },
            };

            if filter.query.trim() != "" &&
                !filename.to_string_lossy().to_lowercase().contains(&filter.query.to_lowercase()) {
                    continue;
            }

            self.songs.push(filename.to_owned());
        }

        self.songs.sort();
    }
}

pub fn spawn_worker(
    context: egui::Context,
    worker_receiver: Receiver<WorkerEvent>,
    ui_sender: Sender<UiEvent>
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut music_list = MusicList::default();

        while let Ok(worker_event) = worker_receiver.recv() {
            match worker_event {
                WorkerEvent::UpdateFilter(filter) => {
                    // Unwrap: If sender is closed, there's nothing we can do
                    ui_sender.send(UiEvent::SetLoading(true)).unwrap();
                    ui_sender.send(UiEvent::UpdateList(music_list.clone())).unwrap();

                    music_list.update(&filter);

                    ui_sender.send(UiEvent::SetLoading(false)).unwrap();
                    ui_sender.send(UiEvent::UpdateList(music_list.clone())).unwrap();
                },

                WorkerEvent::LoadSongImage(path, tag) => {
                    if music_list.song_images.contains_key(&path) {
                        return;
                    }

                    let mut music_list = music_list.clone();
                    music_list.song_images.insert(path.clone(), SongImage { texture_id: None });

                    // Unwrap: If sender is closed, there's nothing we can do
                    ui_sender.send(UiEvent::SetLoading(true)).unwrap();
                    ui_sender.send(UiEvent::UpdateList(music_list.clone())).unwrap();

                    if let Some(image_tag) = tag.pictures().next() {
                        if let Ok(image) = ::image::load_from_memory(&image_tag.data) {
                            let image = image.thumbnail(300, 300);
                            let dimensions = image.dimensions();

                            let egui_image = egui::ColorImage::from_rgba_unmultiplied(
                                [dimensions.0 as usize, dimensions.1 as usize],
                                image.to_rgba8().deref(),
                            );

                            let label = format!("{}", path.display());
                            let handle = context.load_texture(label, egui_image, egui::TextureOptions::default());

                            music_list.song_images.get_mut(&path).unwrap().texture_id = Some(handle.id());
                        }
                    }

                    ui_sender.send(UiEvent::SetLoading(false)).unwrap();
                    ui_sender.send(UiEvent::UpdateList(music_list)).unwrap();
                },
            }
        }
    })
}
