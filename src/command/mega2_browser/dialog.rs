//! Presentation of the existing directory/tag editors; never performs writes.

use super::{BrowserState, Editor, TagEditor};

pub(super) struct Dialog<'a> {
    pub title: &'static str,
    pub context: String,
    pub label: &'static str,
    pub input: Option<&'a str>,
    pub help: &'static str,
    pub error: Option<&'a str>,
}

impl<'a> Dialog<'a> {
    pub fn from_state(state: &'a BrowserState) -> Option<Self> {
        let (title, context, label, input, help, error) = if let Some(panel) = &state.panel {
            let (title, context, label, input, help) = match panel.editor.as_ref()? {
                TagEditor::CreateName(input) => (
                    "New tag",
                    "Root tags / — step 1 of 2".into(),
                    "Tag name",
                    Some(input.as_str()),
                    "Enter continue | Esc cancel",
                ),
                TagEditor::CreateMessage { name, message } => (
                    "New tag",
                    format!("Tag: {name} — step 2 of 2"),
                    "Message (empty = lightweight)",
                    Some(message.as_str()),
                    "Enter create | Esc cancel",
                ),
                TagEditor::ConfirmDelete(name) => (
                    "Delete tag",
                    format!("delete tag '{name}'?"),
                    "Confirm deletion",
                    None,
                    "Enter/y confirm | Esc cancel",
                ),
            };
            (title, context, label, input, help, panel.status.as_deref())
        } else {
            let (title, context, label, input, help) = match state.editor.as_ref()? {
                Editor::CreateDir(input) => (
                    "New directory",
                    format!("Parent: {}", state.path),
                    "Directory name",
                    Some(input.as_str()),
                    "Enter create | Esc cancel",
                ),
                Editor::RenameDir { name, input } => (
                    "Rename directory",
                    format!("Directory: {name}"),
                    "New name",
                    Some(input.as_str()),
                    "Enter confirm | Esc cancel",
                ),
                Editor::ConfirmDelete(name) => (
                    "Delete directory",
                    format!("delete '{name}'?"),
                    "Confirm deletion",
                    None,
                    "Enter/y confirm | Esc cancel",
                ),
            };
            (title, context, label, input, help, state.status.as_deref())
        };
        Some(Self {
            title,
            context,
            label,
            input,
            help,
            error,
        })
    }
}
