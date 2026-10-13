//! Destination selection is modal: browsing must never replace the move source.

use super::{BrowserState, Key};
use crate::internal::protocol::mega2_tree::{ContentType, Listing};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovePicker {
    pub from_parent: String,
    pub name: String,
    pub kind: ContentType,
    /// The top-level monorepo root shared by every valid destination.  Mega2
    /// deliberately refuses writes that would land on `/`, including moves
    /// between roots such as `/artifact` and `/data`.
    pub destination_root: String,
    pub folder: String,
    pub directories: Vec<String>,
    /// Zero selects the current folder; the remaining rows select children.
    pub selection: usize,
    pub confirmed_parent: Option<String>,
    pub error: Option<String>,
    loaded: bool,
}

pub enum MoveChoice {
    Continue,
    Cancel,
    Load(String),
    Submit(String),
}

impl MovePicker {
    pub fn new(parent: String, name: String, kind: ContentType) -> Self {
        let destination_root = Self::top_level_root(&parent);
        Self {
            // Only folders in the source's top-level monorepo root can be
            // valid destinations.  Starting there both avoids a doomed
            // cross-root request and keeps the source immutable while the
            // user browses destinations.
            folder: destination_root.clone(),
            from_parent: parent,
            name,
            kind,
            destination_root,
            directories: Vec::new(),
            selection: 0,
            confirmed_parent: None,
            error: None,
            loaded: false,
        }
    }

    pub fn source_path(&self) -> String {
        BrowserState::child_path(&self.from_parent, &self.name)
    }

    pub fn can_choose_destination(&self) -> bool {
        self.from_parent != "/"
    }

    fn top_level_root(path: &str) -> String {
        path.split('/')
            .find(|component| !component.is_empty())
            .map_or_else(|| "/".to_string(), |component| format!("/{component}"))
    }

    pub fn apply_listing(&mut self, path: String, listing: Listing) {
        self.folder = path;
        self.directories = listing
            .entries
            .into_iter()
            .filter(|entry| entry.content_type == ContentType::Directory)
            .map(|entry| entry.name)
            .collect();
        self.directories.sort_unstable();
        self.selection = 0;
        self.loaded = true;
        self.error = None;
    }

    /// Do not retain a selectable stale destination after a listing request
    /// fails. The user can explicitly reload the current folder or go back.
    pub fn apply_load_error(&mut self, message: String) {
        self.directories.clear();
        self.selection = 0;
        self.confirmed_parent = None;
        self.loaded = false;
        self.error = Some(format!("Could not load destination folders: {message}"));
    }

    pub fn selected_parent(&self) -> String {
        self.selection
            .checked_sub(1)
            .and_then(|index| self.directories.get(index))
            .map_or_else(
                || self.folder.clone(),
                |name| BrowserState::child_path(&self.folder, name),
            )
    }

    fn inside_source(&self, parent: &str) -> bool {
        let source = self.source_path();
        self.kind == ContentType::Directory
            && (parent == source || parent.starts_with(&format!("{source}/")))
    }

    fn in_destination_root(&self, parent: &str) -> bool {
        parent == self.destination_root
            || parent
                .strip_prefix(&self.destination_root)
                .is_some_and(|rest| rest.starts_with('/'))
    }

    pub fn handle_key(&mut self, key: Key) -> MoveChoice {
        if let Some(parent) = &self.confirmed_parent {
            return match key {
                Key::Enter => MoveChoice::Submit(parent.clone()),
                Key::Cancel => {
                    self.confirmed_parent = None;
                    self.error = None;
                    MoveChoice::Continue
                }
                Key::Quit => MoveChoice::Cancel,
                _ => MoveChoice::Continue,
            };
        }
        match key {
            Key::Cancel | Key::Quit => return MoveChoice::Cancel,
            Key::Reload => return MoveChoice::Load(self.folder.clone()),
            Key::Left | Key::Backspace | Key::Home if self.folder != self.destination_root => {
                return MoveChoice::Load(BrowserState::parent_path(&self.folder));
            }
            Key::Left | Key::Backspace | Key::Home => {
                self.error = Some(format!(
                    "Mega2 only allows destinations under {}.",
                    self.destination_root
                ));
            }
            Key::Up | Key::Down => {
                self.selection = if key == Key::Up {
                    self.selection.saturating_sub(1)
                } else {
                    (self.selection + 1).min(self.directories.len())
                };
                self.error = None;
            }
            Key::Enter | Key::Right if self.loaded => {
                let parent = self.selected_parent();
                if !self.can_choose_destination() || !self.in_destination_root(&parent) {
                    self.error = Some(format!(
                        "Mega2 cannot move entries between top-level roots; choose a folder under {}.",
                        self.destination_root
                    ));
                } else if self.inside_source(&parent) {
                    self.error =
                        Some("Cannot move a directory into itself or its descendants.".into());
                } else if key == Key::Right {
                    if self.selection > 0 {
                        return MoveChoice::Load(parent);
                    }
                } else if parent == self.from_parent {
                    self.error = Some("Choose a different destination folder.".into());
                } else {
                    self.confirmed_parent = Some(parent);
                    self.error = None;
                }
            }
            _ => {}
        }
        MoveChoice::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::protocol::mega2_tree::ListingEntry;

    fn listing() -> Listing {
        Listing {
            entries: vec![
                ListingEntry {
                    name: "target".into(),
                    content_type: ContentType::Directory,
                },
                ListingEntry {
                    name: "file.txt".into(),
                    content_type: ContentType::File,
                },
            ],
        }
    }

    #[test]
    fn destination_selection_filters_files_and_requires_a_second_enter() {
        for kind in [ContentType::File, ContentType::Directory] {
            let mut picker = MovePicker::new("/artifact".into(), "source".into(), kind);
            picker.apply_listing("/artifact".into(), listing());
            assert_eq!(picker.directories, ["target"]);
            picker.handle_key(Key::Down);
            assert!(matches!(
                picker.handle_key(Key::Enter),
                MoveChoice::Continue
            ));
            assert_eq!(picker.confirmed_parent.as_deref(), Some("/artifact/target"));
            assert_eq!(picker.source_path(), "/artifact/source");
            assert!(
                matches!(picker.handle_key(Key::Enter), MoveChoice::Submit(parent) if parent == "/artifact/target")
            );
        }
    }

    #[test]
    fn navigation_and_cancel_do_not_change_source_or_submit() {
        let mut picker = MovePicker::new("/artifact".into(), "source".into(), ContentType::File);
        picker.apply_listing("/artifact".into(), listing());
        picker.handle_key(Key::Down);
        assert!(
            matches!(picker.handle_key(Key::Right), MoveChoice::Load(parent) if parent == "/artifact/target")
        );
        picker.apply_listing("/artifact/target".into(), listing());
        picker.handle_key(Key::Enter);
        assert!(matches!(
            picker.handle_key(Key::Cancel),
            MoveChoice::Continue
        ));
        assert!(picker.confirmed_parent.is_none());
        assert!(matches!(picker.handle_key(Key::Cancel), MoveChoice::Cancel));
        assert_eq!(picker.source_path(), "/artifact/source");
    }

    #[test]
    fn unchanged_parent_and_own_subtree_are_refused() {
        let mut picker =
            MovePicker::new("/artifact".into(), "target".into(), ContentType::Directory);
        picker.apply_listing("/artifact".into(), listing());
        picker.handle_key(Key::Enter);
        assert!(picker.error.is_some());
        picker.handle_key(Key::Down);
        picker.handle_key(Key::Enter);
        assert!(picker.confirmed_parent.is_none());
        assert!(picker.error.as_deref().unwrap().contains("itself"));
        assert!(matches!(
            picker.handle_key(Key::Right),
            MoveChoice::Continue
        ));
        picker.apply_listing("/artifact/target/child".into(), listing());
        picker.handle_key(Key::Enter);
        assert!(picker.confirmed_parent.is_none());
    }

    #[test]
    fn picker_never_offers_a_cross_top_level_destination() {
        let mut picker = MovePicker::new(
            "/artifact/src".into(),
            "readme.txt".into(),
            ContentType::File,
        );
        assert_eq!(picker.destination_root, "/artifact");
        assert_eq!(picker.folder, "/artifact");

        // The normal browser never loads `/data` here.  This guard keeps the
        // policy intact if a future caller accidentally supplies one.
        picker.apply_listing(
            "/data".into(),
            Listing {
                entries: Vec::new(),
            },
        );
        picker.handle_key(Key::Enter);
        assert!(picker.confirmed_parent.is_none());
        assert!(
            picker
                .error
                .as_deref()
                .is_some_and(|error| error.contains("top-level roots"))
        );
    }

    #[test]
    fn failed_destination_load_clears_stale_choices_until_explicit_reload() {
        let mut picker = MovePicker::new("/artifact".into(), "source".into(), ContentType::File);
        picker.apply_listing("/artifact".into(), listing());
        picker.handle_key(Key::Down);
        assert!(matches!(
            picker.handle_key(Key::Right),
            MoveChoice::Load(path) if path == "/artifact/target"
        ));

        picker.apply_load_error("request timed out".into());
        assert!(picker.directories.is_empty());
        assert_eq!(picker.selection, 0);
        assert!(picker.confirmed_parent.is_none());
        assert!(matches!(
            picker.handle_key(Key::Enter),
            MoveChoice::Continue
        ));
        assert!(picker.confirmed_parent.is_none());
        assert!(matches!(
            picker.handle_key(Key::Reload),
            MoveChoice::Load(path) if path == "/artifact"
        ));
    }
}
