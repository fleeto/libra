//! Interactive tree/preview state, separate from the existing mutation editors.

use super::{ActionResult, BrowserState, Key, highlight::Highlighted, tree::DirectoryTree};
use crate::{internal::protocol::mega2_tree::ContentType, utils::error::CliResult};

#[derive(Default)]
pub struct Preview {
    pub path: Option<String>,
    pub lines: Vec<String>,
    pub message: String,
    pub top: usize,
    pub left: usize,
    pub max_width: usize,
    pub highlighted: Highlighted,
}

pub enum ExplorerAction {
    Core(ActionResult),
    LoadDirectory(String),
    LoadRoot(String),
    /// Load a folder into the modal destination picker without replacing a
    /// branch in the visible source tree.
    LoadMoveDestination(String),
}

pub struct Explorer {
    pub tree: DirectoryTree,
    pub preview: Preview,
    pub content_focus: bool,
    pub pending_preview: Option<String>,
    pub color_enabled: bool,
}

impl Explorer {
    fn from_browser_action(action: ActionResult) -> ExplorerAction {
        match action {
            ActionResult::LoadMoveDestination { path } => ExplorerAction::LoadMoveDestination(path),
            action => ExplorerAction::Core(action),
        }
    }

    pub fn new(state: &BrowserState) -> CliResult<Self> {
        let mut view = Self {
            tree: DirectoryTree::new(state.path.clone(), &state.entries)?,
            preview: Preview::default(),
            content_focus: false,
            pending_preview: None,
            color_enabled: true,
        };
        view.tree.selection = state.selection.min(view.tree.rows.len().saturating_sub(1));
        view.refresh_preview(false);
        Ok(view)
    }

    pub fn refresh_preview(&mut self, force: bool) {
        let selected = self.tree.selected();
        let path = selected.map(|row| row.path.clone());
        let is_file = selected.is_some_and(|row| row.kind == ContentType::File);
        if !force && path == self.preview.path {
            return;
        }
        self.preview = Preview {
            path: path.clone(),
            message: if is_file {
                "Loading file..."
            } else {
                "Select a file to preview. Enter or Right expands a directory."
            }
            .into(),
            ..Preview::default()
        };
        self.pending_preview = if is_file { path } else { None };
    }

    pub fn apply_preview(&mut self, result: CliResult<String>) {
        self.preview.lines.clear();
        self.preview.max_width = 0;
        self.preview.highlighted = Highlighted::default();
        match result {
            Ok(text) => {
                self.preview.lines = text
                    .lines()
                    .map(|line| super::layout::safe_text(&line.replace('\t', "    ")))
                    .collect();
                self.preview.max_width = self
                    .preview
                    .lines
                    .iter()
                    .map(|line| unicode_width::UnicodeWidthStr::width(line.as_str()))
                    .max()
                    .unwrap_or(0);
                self.preview.message = if self.preview.lines.is_empty() {
                    "(empty file)".into()
                } else {
                    String::new()
                };
                if self.color_enabled {
                    self.preview.highlighted = Highlighted::for_file(
                        self.preview.path.as_deref().unwrap_or(""),
                        &self.preview.lines,
                    );
                }
            }
            Err(error) => {
                self.preview.message = format!("Preview unavailable: {}", error.message())
            }
        }
    }

    pub fn handle_key(
        &mut self,
        state: &mut BrowserState,
        key: Key,
        page_rows: usize,
    ) -> ExplorerAction {
        // The move picker owns its source snapshot.  Route it before the
        // ordinary q/Esc/tree handlers so destination navigation can never
        // open, collapse, or replace a branch in the source tree.
        if state.move_picker.is_some() {
            return Self::from_browser_action(state.handle_key(key));
        }
        if state.editor.is_some() || state.panel.is_some() {
            return ExplorerAction::Core(state.handle_key(key));
        }
        if matches!(key, Key::Quit | Key::Cancel | Key::Tags) {
            return ExplorerAction::Core(state.handle_key(key));
        }
        if key == Key::Tab {
            self.content_focus = !self.content_focus;
        } else if matches!(key, Key::PageUp | Key::PageDown) || self.content_focus {
            match key {
                Key::Up => self.preview.top = self.preview.top.saturating_sub(1),
                Key::Down => {
                    self.preview.top =
                        (self.preview.top + 1).min(self.preview.lines.len().saturating_sub(1))
                }
                Key::PageUp => self.preview.top = self.preview.top.saturating_sub(page_rows),
                Key::PageDown => {
                    self.preview.top = (self.preview.top + page_rows)
                        .min(self.preview.lines.len().saturating_sub(1))
                }
                Key::Left => self.preview.left = self.preview.left.saturating_sub(4),
                Key::Right => {
                    self.preview.left =
                        (self.preview.left + 4).min(self.preview.max_width.saturating_sub(1))
                }
                Key::Reload => self.refresh_preview(true),
                _ => {}
            }
        } else {
            match key {
                Key::Up | Key::Down => {
                    self.tree.move_selection(key == Key::Down);
                    state.status = None;
                }
                Key::Enter | Key::Right => {
                    if let Some(row) = self.tree.selected() {
                        if row.kind == ContentType::File {
                            self.content_focus = true;
                        } else if !row.expanded {
                            return ExplorerAction::LoadDirectory(row.path.clone());
                        } else if key == Key::Enter {
                            self.tree.collapse(&row.path.clone());
                        } else if self
                            .tree
                            .rows
                            .get(self.tree.selection + 1)
                            .is_some_and(|child| child.depth > row.depth)
                        {
                            self.tree.move_selection(true);
                        }
                    }
                }
                Key::Left | Key::Backspace | Key::Home => {
                    if let Some(row) = self.tree.selected() {
                        if row.expanded {
                            self.tree.collapse(&row.path.clone());
                        } else if row.depth == 0 && self.tree.root != "/" {
                            return ExplorerAction::LoadRoot(BrowserState::parent_path(
                                &self.tree.root,
                            ));
                        } else {
                            self.tree.select_parent();
                        }
                    } else if self.tree.root != "/" {
                        return ExplorerAction::LoadRoot(BrowserState::parent_path(
                            &self.tree.root,
                        ));
                    }
                }
                Key::Reload => {
                    if let Some(row) = self.tree.selected() {
                        if row.kind == ContentType::Directory {
                            return ExplorerAction::LoadDirectory(row.path.clone());
                        }
                        self.refresh_preview(true);
                    } else {
                        return ExplorerAction::LoadDirectory(self.tree.root.clone());
                    }
                }
                _ => {
                    self.tree.sync_selection(state);
                    if key == Key::Create
                        && let Some(row) = self.tree.selected()
                        && row.kind == ContentType::Directory
                    {
                        // Creation goes inside the selected directory; other
                        // mutations operate on that row in its parent branch.
                        state.path = row.path.clone();
                    }
                    return Self::from_browser_action(state.handle_key(key));
                }
            }
        }
        self.tree.sync_selection(state);
        self.refresh_preview(false);
        ExplorerAction::Core(ActionResult::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::protocol::mega2_tree::{Listing, ListingEntry, Mega2TreeSession};

    fn setup() -> (BrowserState, Explorer) {
        let mut state = BrowserState::new(
            Mega2TreeSession::new("http://localhost:9000").unwrap(),
            "/",
            None,
        )
        .unwrap();
        state.entries = vec![
            ("dir".into(), ContentType::Directory),
            ("file".into(), ContentType::File),
        ];
        let view = Explorer::new(&state).unwrap();
        (state, view)
    }

    #[test]
    fn file_selection_loads_once_and_directory_selection_clears_preview() {
        let (mut state, mut view) = setup();
        view.handle_key(&mut state, Key::Down, 10);
        assert_eq!(view.pending_preview.take().as_deref(), Some("/file"));
        view.apply_preview(Ok("hello".into()));
        view.handle_key(&mut state, Key::Down, 10);
        assert!(view.pending_preview.is_none());
        view.handle_key(&mut state, Key::Up, 10);
        assert!(view.preview.lines.is_empty());
        assert!(view.pending_preview.is_none());
    }

    #[test]
    fn preview_focus_scrolls_without_moving_tree_or_opening_write_editors() {
        let (mut state, mut view) = setup();
        view.handle_key(&mut state, Key::Down, 10);
        view.apply_preview(Ok((0..40).map(|n| format!("line {n}\n")).collect()));
        view.pending_preview = None;
        view.handle_key(&mut state, Key::Tab, 10);
        for key in [
            Key::PageDown,
            Key::Create,
            Key::Delete,
            Key::Move,
            Key::Rename,
        ] {
            view.handle_key(&mut state, key, 10);
        }
        assert_eq!(view.preview.top, 10);
        assert_eq!(view.tree.selection, 1);
        assert!(state.editor.is_none());
        assert!(view.pending_preview.is_none());
    }

    #[test]
    fn nested_delete_confirmation_targets_the_selected_branch() {
        let (mut state, mut view) = setup();
        view.tree
            .replace_children("/dir", &[("child".into(), ContentType::Directory)])
            .unwrap();
        view.tree.selection = 1;
        view.handle_key(&mut state, Key::Delete, 10);
        assert!(
            matches!(view.handle_key(&mut state, Key::Enter, 10), ExplorerAction::Core(ActionResult::DeleteDirectory { parent, name }) if parent == "/dir" && name == "child")
        );
    }

    #[test]
    fn create_targets_selected_directory_collapsed_expanded_or_nested() {
        for (expanded, nested) in [(false, false), (true, false), (true, true)] {
            let (mut state, mut view) = setup();
            if expanded {
                view.tree
                    .replace_children("/dir", &[("child".into(), ContentType::Directory)])
                    .unwrap();
            }
            if nested {
                view.tree.selection = 1;
            }
            let expected_parent = if nested { "/dir/child" } else { "/dir" };
            view.handle_key(&mut state, Key::Create, 10);
            assert_eq!(state.path, expected_parent, "dialog parent");
            for ch in "rust".chars() {
                view.handle_key(&mut state, Key::Other(ch), 10);
            }
            assert!(
                matches!(view.handle_key(&mut state, Key::Enter, 10), ExplorerAction::Core(ActionResult::CreateDirectory { parent, name }) if parent == expected_parent && name == "rust")
            );
        }
    }

    #[test]
    fn create_in_empty_tree_uses_visible_root_and_file_selection_is_inert() {
        let mut state = BrowserState::new(
            Mega2TreeSession::new("http://localhost:9000").unwrap(),
            "/artifact",
            None,
        )
        .unwrap();
        let mut view = Explorer::new(&state).unwrap();
        view.handle_key(&mut state, Key::Create, 10);
        view.handle_key(&mut state, Key::Other('x'), 10);
        assert!(
            matches!(view.handle_key(&mut state, Key::Enter, 10), ExplorerAction::Core(ActionResult::CreateDirectory { parent, name }) if parent == "/artifact" && name == "x")
        );

        let (mut state, mut view) = setup();
        view.handle_key(&mut state, Key::Down, 10);
        view.handle_key(&mut state, Key::Create, 10);
        assert!(state.editor.is_none());
        assert!(
            state
                .status
                .as_deref()
                .unwrap()
                .contains("select a directory")
        );
    }

    #[test]
    fn cancelling_create_restores_selected_directory_as_mutation_operand() {
        let (mut state, mut view) = setup();
        for key in [Key::Delete, Key::Rename] {
            view.handle_key(&mut state, Key::Create, 10);
            view.handle_key(&mut state, Key::Cancel, 10);
            assert_eq!(view.tree.selected().unwrap().path, "/dir");
            view.handle_key(&mut state, key, 10);
            assert_eq!(state.path, "/", "operand parent after {key:?}");
            let action = view.handle_key(&mut state, Key::Enter, 10);
            match action {
                ExplorerAction::Core(ActionResult::DeleteDirectory { parent, name }) => {
                    assert_eq!((parent.as_str(), name.as_str()), ("/", "dir"));
                }
                ExplorerAction::Core(ActionResult::MoveEntry {
                    from_parent,
                    from_name,
                    to_parent,
                    to_name,
                    kind: ContentType::Directory,
                }) => {
                    assert_eq!((from_parent.as_str(), from_name.as_str()), ("/", "dir"));
                    assert_eq!((to_parent.as_str(), to_name.as_str()), ("/", "dir"));
                }
                _ => panic!("expected directory mutation"),
            }
        }
    }

    #[test]
    fn move_picker_freezes_the_nested_source_and_right_only_browses_destination() {
        let mut state = BrowserState::new(
            Mega2TreeSession::new("http://localhost:9000").unwrap(),
            "/artifact",
            None,
        )
        .unwrap();
        state.entries = vec![
            ("readme.txt".into(), ContentType::File),
            ("destination".into(), ContentType::Directory),
        ];
        let mut view = Explorer::new(&state).unwrap();

        assert!(matches!(
            view.handle_key(&mut state, Key::Move, 10),
            ExplorerAction::LoadMoveDestination(ref path) if path == "/artifact"
        ));
        let picker = state.move_picker.as_ref().expect("picker");
        assert_eq!(picker.source_path(), "/artifact/readme.txt");
        assert_eq!(picker.kind, ContentType::File);

        state.move_picker.as_mut().expect("picker").apply_listing(
            "/artifact".into(),
            Listing {
                entries: vec![ListingEntry {
                    name: "destination".into(),
                    content_type: ContentType::Directory,
                }],
            },
        );
        assert!(matches!(
            view.handle_key(&mut state, Key::Down, 10),
            ExplorerAction::Core(ActionResult::Continue)
        ));
        assert!(matches!(
            view.handle_key(&mut state, Key::Right, 10),
            ExplorerAction::LoadMoveDestination(ref path) if path == "/artifact/destination"
        ));
        assert_eq!(
            view.tree.selected().map(|row| row.path.as_str()),
            Some("/artifact/readme.txt")
        );
        assert_eq!(state.path, "/artifact");

        state.move_picker.as_mut().expect("picker").apply_listing(
            "/artifact/destination".into(),
            Listing {
                entries: Vec::new(),
            },
        );
        assert!(matches!(
            view.handle_key(&mut state, Key::Enter, 10),
            ExplorerAction::Core(ActionResult::Continue)
        ));
        assert!(matches!(
            view.handle_key(&mut state, Key::Enter, 10),
            ExplorerAction::Core(ActionResult::MoveEntry { ref from_parent, ref from_name, ref to_parent, ref to_name, kind: ContentType::File })
                if from_parent == "/artifact"
                    && from_name == "readme.txt"
                    && to_parent == "/artifact/destination"
                    && to_name == "readme.txt"
        ));
    }
}
