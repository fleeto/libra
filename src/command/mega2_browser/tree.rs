//! Bounded, flattened directory tree. Only explicitly expanded nodes are loaded.

use super::BrowserState;
use crate::{
    internal::protocol::mega2_tree::ContentType,
    utils::error::{CliError, CliResult, StableErrorCode},
};

const MAX_TREE_ROWS: usize = 8_192;
const MAX_TREE_BYTES: usize = 4 << 20;

#[derive(Debug, Clone)]
pub struct TreeRow {
    pub path: String,
    pub name: String,
    pub kind: ContentType,
    pub depth: usize,
    pub expanded: bool,
}

pub struct DirectoryTree {
    pub root: String,
    pub rows: Vec<TreeRow>,
    pub selection: usize,
}

impl DirectoryTree {
    pub fn new(root: String, entries: &[(String, ContentType)]) -> CliResult<Self> {
        let mut tree = Self {
            root,
            rows: Vec::new(),
            selection: 0,
        };
        tree.replace_children(&tree.root.clone(), entries)?;
        Ok(tree)
    }

    pub fn selected(&self) -> Option<&TreeRow> {
        self.rows.get(self.selection)
    }

    fn children_range(&self, parent: &str) -> Option<(usize, usize, usize)> {
        if parent == self.root {
            return Some((0, self.rows.len(), 0));
        }
        let index = self.rows.iter().position(|row| row.path == parent)?;
        let depth = self.rows[index].depth;
        let end = self.rows[index + 1..]
            .iter()
            .position(|row| row.depth <= depth)
            .map_or(self.rows.len(), |offset| index + 1 + offset);
        Some((index + 1, end, depth + 1))
    }

    /// Replace one loaded branch atomically; errors leave the old tree intact.
    pub fn replace_children(
        &mut self,
        parent: &str,
        entries: &[(String, ContentType)],
    ) -> CliResult<()> {
        let Some((start, end, depth)) = self.children_range(parent) else {
            return Ok(());
        };
        let retained_bytes: usize = self.rows[..start]
            .iter()
            .chain(&self.rows[end..])
            .map(|row| row.path.len() + row.name.len())
            .sum();
        let incoming_bytes: usize = entries
            .iter()
            .map(|(name, _)| parent.len() + 1 + 2 * name.len())
            .sum();
        if self.rows.len() - (end - start) + entries.len() > MAX_TREE_ROWS
            || retained_bytes.saturating_add(incoming_bytes) > MAX_TREE_BYTES
            || depth > 128
        {
            return Err(CliError::fatal(
                "directory tree limit reached; collapse a branch before expanding another",
            )
            .with_stable_code(StableErrorCode::Unsupported));
        }
        let selected = self.selected().map(|row| row.path.clone());
        self.rows.splice(
            start..end,
            entries.iter().map(|(name, kind)| TreeRow {
                path: BrowserState::child_path(parent, name),
                name: name.clone(),
                kind: *kind,
                depth,
                expanded: false,
            }),
        );
        if parent != self.root
            && let Some(row) = self.rows.get_mut(start - 1)
        {
            row.expanded = true;
        }
        self.selection = selected
            .and_then(|path| self.rows.iter().position(|row| row.path == path))
            .unwrap_or(self.selection.min(self.rows.len().saturating_sub(1)));
        Ok(())
    }

    pub fn collapse(&mut self, path: &str) {
        if path == self.root {
            return;
        }
        if let Some((start, end, _)) = self.children_range(path) {
            let selected = self.selected().map(|row| row.path.clone());
            self.rows.drain(start..end);
            if let Some(row) = self.rows.get_mut(start - 1) {
                row.expanded = false;
            }
            self.selection = selected
                .and_then(|path| self.rows.iter().position(|row| row.path == path))
                .unwrap_or(start - 1);
        }
    }

    pub fn move_selection(&mut self, down: bool) {
        self.selection = if down {
            (self.selection + 1).min(self.rows.len().saturating_sub(1))
        } else {
            self.selection.saturating_sub(1)
        };
    }

    pub fn select_parent(&mut self) {
        if let Some(row) = self.selected() {
            let parent = BrowserState::parent_path(&row.path);
            if let Some(index) = self.rows.iter().position(|row| row.path == parent) {
                self.selection = index;
            }
        }
    }

    /// Adapt the selected row to the existing single-directory mutation editor.
    /// A nested row must never inherit the root's mutation target.
    pub fn sync_selection(&self, state: &mut BrowserState) {
        let parent = self
            .selected()
            .map(|row| BrowserState::parent_path(&row.path))
            .unwrap_or_else(|| self.root.clone());
        state.path = parent.clone();
        state.entries = self
            .rows
            .iter()
            .filter(|row| BrowserState::parent_path(&row.path) == parent)
            .map(|row| (row.name.clone(), row.kind))
            .collect();
        state.selection = self
            .selected()
            .and_then(|selected| {
                state
                    .entries
                    .iter()
                    .position(|(name, _)| name == &selected.name)
            })
            .unwrap_or(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::protocol::mega2_tree::Mega2TreeSession;

    #[test]
    fn expanded_branches_keep_siblings_and_collapse_descendants() {
        let mut tree = DirectoryTree::new(
            "/".into(),
            &[
                ("a".into(), ContentType::Directory),
                ("z".into(), ContentType::File),
            ],
        )
        .unwrap();
        tree.replace_children("/a", &[("nested".into(), ContentType::Directory)])
            .unwrap();
        tree.replace_children("/a/nested", &[("file".into(), ContentType::File)])
            .unwrap();
        assert_eq!(
            tree.rows
                .iter()
                .map(|row| row.path.as_str())
                .collect::<Vec<_>>(),
            ["/a", "/a/nested", "/a/nested/file", "/z"]
        );
        tree.selection = 2;
        tree.collapse("/a");
        assert_eq!(tree.rows.len(), 2);
        assert_eq!(tree.selected().unwrap().path, "/a");
        assert!(!tree.rows[0].expanded);
    }

    #[test]
    fn nested_mutation_uses_selected_rows_actual_parent() {
        let mut tree =
            DirectoryTree::new("/".into(), &[("a".into(), ContentType::Directory)]).unwrap();
        tree.replace_children("/a", &[("child".into(), ContentType::Directory)])
            .unwrap();
        tree.selection = 1;
        let mut state = BrowserState::new(
            Mega2TreeSession::new("http://localhost:9000").unwrap(),
            "/",
            None,
        )
        .unwrap();
        tree.sync_selection(&mut state);
        assert_eq!(state.path, "/a");
        assert_eq!(state.entries[state.selection].0, "child");
    }

    #[test]
    fn oversized_branch_is_rejected_without_losing_selection() {
        let mut tree =
            DirectoryTree::new("/".into(), &[("a".into(), ContentType::Directory)]).unwrap();
        let oversized = vec![("long-name".into(), ContentType::File); MAX_TREE_ROWS];
        assert!(tree.replace_children("/a", &oversized).is_err());
        assert_eq!(tree.rows.len(), 1);
        assert!(!tree.rows[0].expanded);
    }
}
