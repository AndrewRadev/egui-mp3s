use std::path::{Path, PathBuf};
use egui_mp3s::music::{MusicList, MusicFilter};

fn test_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let base_path = PathBuf::from(file!()).parent().unwrap().join("fixtures");

    base_path.join(path)
}

#[test]
fn test_listing_songs() {
    let mut list = MusicList::default();
    let mut filter = MusicFilter {
        root_dir: format!("{}", test_path("dir1").display()),
        query: String::new(),
    };

    list.update(&filter);
    assert_eq!(list.songs, [PathBuf::from("a.mp3"), PathBuf::from("b.mp3"), PathBuf::from("c.mp3")]);

    filter.query = String::from("b");

    list.update(&filter);
    assert_eq!(list.songs, [PathBuf::from("b.mp3")]);

    // Changing root dir doesn't change the contents after update
    filter.root_dir = format!("{}", test_path("nonexistent").display());
    list.update(&filter);
    assert_eq!(list.songs, [PathBuf::from("b.mp3")]);
}

#[test]
fn test_listing_nested_songs() {
    let mut list = MusicList::default();
    let mut filter = MusicFilter {
        root_dir: format!("{}", test_path("dir2").display()),
        query: String::new(),
    };

    list.update(&filter);
    assert_eq!(list.songs, [PathBuf::from("a.mp3"), PathBuf::from("b.mp3"), PathBuf::from("nested/b.mp3")]);

    filter.query = String::from("b");

    list.update(&filter);
    assert_eq!(list.songs, [PathBuf::from("b.mp3"), PathBuf::from("nested/b.mp3")]);
}
