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
}

pub struct Explorer {
    pub tree: DirectoryTree,
    pub preview: Preview,
    pub content_focus: bool,
    pub pending_preview: Option<String>,
    pub color_enabled: bool,
}

impl Explorer {
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
                    return ExplorerAction::Core(state.handle_key(key));
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
    use crate::internal::protocol::mega2_tree::Mega2TreeSession;

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
}
