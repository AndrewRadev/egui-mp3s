use std::path::PathBuf;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;

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
    pub loading: bool,
    pub songs: Vec<PathBuf>,
}

impl MusicList {
    pub fn update(&mut self, filter: &MusicFilter) {
        let dir_entries = match std::fs::read_dir(&filter.root_dir) {
            Ok(dir_entries) => dir_entries,
            _ => return,
        };

        self.songs.clear();

        for entry in dir_entries {
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
    filter_receiver: Receiver<MusicFilter>,
    list_sender: Sender<MusicList>
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut music_list = MusicList::default();

        while let Ok(filter) = filter_receiver.recv() {
            music_list.loading = true;
            // Unwrap: If sender is closed, there's nothing we can do
            list_sender.send(music_list.clone()).unwrap();

            music_list.update(&filter);

            music_list.loading = false;
            // Unwrap: If sender is closed, there's nothing we can do
            list_sender.send(music_list.clone()).unwrap();
        }
    })
}
