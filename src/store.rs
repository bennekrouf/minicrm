//! Where the cockpit is kept: one JSON file in the app's data folder,
//! written again after every change.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use crate::model::Cockpit;
use crate::seed;

const FILE: &str = "cockpit.json";
/// What a file that could not be read is renamed to, so the next save does
/// not write over it.
const UNREADABLE: &str = "cockpit.unreadable.json";

/// The app's data folder.
pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("minicrm")
}

/// The file itself, to show it in Finder or Explorer.
pub fn file() -> PathBuf {
    data_dir().join(FILE)
}

/// The saved cockpit; the examples on a first launch.
pub fn load() -> Cockpit {
    load_from(&data_dir(), chrono::Local::now().date_naive())
}

pub fn save(cockpit: &Cockpit) {
    save_to(&data_dir(), cockpit);
}

fn load_from(dir: &Path, today: NaiveDate) -> Cockpit {
    let path = dir.join(FILE);
    let Ok(content) = std::fs::read_to_string(&path) else {
        return seed::examples(today);
    };
    match serde_json::from_str(&content) {
        Ok(cockpit) => cockpit,
        Err(_) => {
            // Someone's notes are in there, even if this version cannot read
            // them: set the file aside before the examples take its place.
            let _ = std::fs::rename(&path, dir.join(UNREADABLE));
            seed::examples(today)
        }
    }
}

fn save_to(dir: &Path, cockpit: &Cockpit) {
    let _ = std::fs::create_dir_all(dir);
    if let Ok(json) = serde_json::to_string_pretty(cockpit) {
        let _ = write_atomic(&dir.join(FILE), json.as_bytes());
    }
}

/// Writes through a temporary file in the same folder, then renames it into
/// place: a crash or a full disk part-way leaves the previous file whole.
fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Sheet;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 11).unwrap()
    }

    /// A folder of its own for each test, under the system's temporary one.
    fn folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("minicrm-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_first_launch_opens_on_the_examples() {
        let dir = folder("first");
        assert_eq!(load_from(&dir, today()), seed::examples(today()));
    }

    #[test]
    fn what_is_saved_comes_back() {
        let dir = folder("roundtrip");
        let mut cockpit = seed::examples(today());
        let id = cockpit.add(Sheet::Signals, today());
        cockpit.set(Sheet::Signals, id, "signal", "Un « test » accentué".into());
        save_to(&dir, &cockpit);
        assert_eq!(load_from(&dir, today()), cockpit);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_emptied_cockpit_stays_empty() {
        let dir = folder("empty");
        save_to(&dir, &Cockpit::default());
        assert_eq!(load_from(&dir, today()), Cockpit::default());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_unreadable_file_is_set_aside_not_overwritten() {
        let dir = folder("broken");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE), "{ not json").unwrap();
        let cockpit = load_from(&dir, today());
        assert_eq!(cockpit, seed::examples(today()));
        save_to(&dir, &cockpit);
        assert_eq!(
            std::fs::read_to_string(dir.join(UNREADABLE)).unwrap(),
            "{ not json"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
