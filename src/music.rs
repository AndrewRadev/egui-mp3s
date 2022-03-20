use std::path::PathBuf;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;

use walkdir::WalkDir;

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
                }
            }
        }
    })
}
