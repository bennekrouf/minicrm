//! The media library PolishCast, TrackCut and PublishKit share: every video
//! recorded, track cut and post written, and where each one stands, from
//! planned to published (`desktop_kit::library`). MiniCRM only reads it. A
//! post that goes out is logged in Field progress
//! ([`crate::model::Cockpit::log_posts`]), and an action can name the video
//! or post it was about.

use std::sync::{Mutex, PoisonError};

use desktop_kit::library::{self, Item, Kind, Status};

use crate::model::Tone;

/// The library as last read. The window keeps it current as it follows the
/// library; `minicrm mcp` reads it again at a call when an app changed it.
static ITEMS: Mutex<Vec<Item>> = Mutex::new(Vec::new());
/// What [`library::stamp`] was when `minicrm mcp` last read it.
static STAMP: Mutex<Option<u64>> = Mutex::new(None);

pub fn set(items: Vec<Item>) {
    *ITEMS.lock().unwrap_or_else(PoisonError::into_inner) = items;
}

/// Reads the library again if an app changed it since the last time.
pub fn refresh() {
    let now = library::stamp();
    let mut seen = STAMP.lock().unwrap_or_else(PoisonError::into_inner);
    if *seen != Some(now) {
        *seen = Some(now);
        set(library::load_all());
    }
}

/// Every item, newest first.
pub fn items() -> Vec<Item> {
    ITEMS.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

pub fn find(id: &str) -> Option<Item> {
    if id.is_empty() {
        return None;
    }
    ITEMS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .find(|i| i.id == id)
        .cloned()
}

/// A blog or social post, which is what goes out to people; a video goes
/// out inside one.
pub fn is_post(item: &Item) -> bool {
    matches!(item.kind, Kind::Blog | Kind::Social)
}

/// Where a post went out, as people say it: `LinkedIn`, `Blog`.
pub fn place(item: &Item) -> String {
    match (item.kind, item.network.as_deref()) {
        (Kind::Blog, _) => "Blog".into(),
        (_, Some("linkedin")) => "LinkedIn".into(),
        (_, Some("x")) => "X".into(),
        (_, Some("instagram")) => "Instagram".into(),
        (_, Some("tiktok")) => "TikTok".into(),
        (_, Some(other)) => other.to_string(),
        (kind, None) => kind.label().into(),
    }
}

/// What an item is, as people say it: `LinkedIn post`, `Blog post`, `Video`,
/// or `Social post` for a post planned before its network is chosen.
pub fn what(item: &Item) -> String {
    match item.kind {
        Kind::Social if item.network.is_some() => format!("{} post", place(item)),
        kind => kind.label().into(),
    }
}

/// The colour of a status, as the tables colour theirs: green once out.
pub fn tone(status: Status) -> Tone {
    match status {
        Status::Planned => Tone::Neutral,
        Status::Recording | Status::Polishing => Tone::Soft,
        Status::Ready | Status::Queued => Tone::Warn,
        Status::Posted | Status::Published => Tone::Good,
    }
}

/// How a cell naming an item reads: its title, or that it is gone.
pub fn title(id: &str) -> String {
    if id.is_empty() {
        return String::new();
    }
    find(id).map_or_else(|| "No longer in the library".into(), |i| i.title)
}

/// The item a value names: its id, or its title whatever the case.
pub fn named<'a>(items: &'a [Item], value: &str) -> Result<&'a Item, String> {
    let value = value.trim();
    if let Some(item) = items.iter().find(|i| i.id == value) {
        return Ok(item);
    }
    let lower = value.to_lowercase();
    let same: Vec<&Item> = items
        .iter()
        .filter(|i| i.title.trim().to_lowercase() == lower)
        .collect();
    match same.as_slice() {
        [one] => Ok(one),
        [] => Err(format!(
            "nothing in the media library is called {value}; give the item's id \
             (overview lists this week's, PublishKit's or PolishCast's list_media all of them)"
        )),
        many => {
            let ids: Vec<String> = many
                .iter()
                .map(|i| format!("{} ({}, {})", i.id, what(i), i.day()))
                .collect();
            Err(format!(
                "{} items are called {value}: give one id of {}",
                many.len(),
                ids.join(", ")
            ))
        }
    }
}

/// Opens an item where it was made: in PolishCast, TrackCut or PublishKit,
/// or, for a file dropped in the library by hand, the file itself.
pub fn open(item: &Item) -> Result<(), String> {
    if item.app == "library" || item.open.is_empty() {
        return match item.main_file() {
            Some(f) => desktop_kit::launch::open_file(&f.path),
            None => Err("It has no file to open yet.".into()),
        };
    }
    desktop_kit::handoff::send(
        &item.app,
        "open",
        "minicrm",
        serde_json::json!({ "open": item.open }),
    )?;
    desktop_kit::launch::open_app(&item.app)
}

/// The app an item opens in, by name.
pub fn app_name(item: &Item) -> &str {
    if item.app == "library" {
        return "its app";
    }
    desktop_kit::launch::name(&item.app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_post_reads_as_where_it_goes_out() {
        let mut post = Item::new("p", "publishkit", Kind::Social, "Launch", Status::Queued);
        post.network = Some("linkedin".into());
        assert_eq!(what(&post), "LinkedIn post");

        let planned = Item::new("plan:1", "library", Kind::Social, "Launch", Status::Planned);
        assert_eq!(what(&planned), "Social post");

        let blog = Item::new("b", "publishkit", Kind::Blog, "Why", Status::Planned);
        assert_eq!(what(&blog), "Blog post");
    }
}
