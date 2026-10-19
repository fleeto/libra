//! User-level Grok hook files.
//!
//! Only matcher groups carrying the `libra capture` marker are rewritten.
//! A file with no marker keeps its original bytes. The installed command is
//! the caller-supplied Libra command; this module does not start it.

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use serde_json::{Map, Value};

pub const LIBRA_CAPTURE_MARKER: &str = "libra capture";
const OWNED_FILE_NAME: &str = "libra-grok-capture.json";
const BOUNDARY_EVENTS: &[&str] = &["Stop", "StopCancelled", "StopFailure", "PreCompact"];

#[derive(Debug, thiserror::Error)]
pub enum GrokHookSettingsError {
    #[error("refusing to install a command whose program is grok")]
    GrokCommand,
    #[error("failed to update '{path}': {detail}")]
    Io { path: String, detail: String },
    #[error("malformed hook file '{path}': {detail}")]
    Malformed { path: String, detail: String },
}

pub fn grok_user_hooks_dir() -> Option<PathBuf> {
    let home = std::env::var_os("GROK_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|path| path.join(".grok")))?;
    Some(home.join("hooks"))
}

pub fn grok_hook_command(binary: &str, event: &str) -> String {
    let verb = match event {
        "PreCompact" => "compaction",
        "SessionEnd" => "session-end",
        _ => "stop",
    };
    format!("{binary} agent hooks grok {verb}")
}

pub fn install_grok_boundary_hooks(
    hooks_dir: &Path,
    binary: &str,
) -> Result<(), GrokHookSettingsError> {
    if command_program_is_grok(binary) {
        return Err(GrokHookSettingsError::GrokCommand);
    }
    fs::create_dir_all(hooks_dir).map_err(|error| io_error(hooks_dir, error))?;
    for path in json_files(hooks_dir)? {
        let bytes = fs::read(&path).map_err(|error| io_error(&path, error))?;
        if !bytes_have_marker(&bytes) {
            continue;
        }
        let updated = rewrite_marked(&path, &bytes, |value| {
            update_marked_commands_for_binary(value, binary);
        })?;
        atomic_write(&path, &updated)?;
    }
    let owned = hooks_dir.join(OWNED_FILE_NAME);
    if !owned.exists() {
        atomic_write(&owned, &owned_file_bytes_for_binary(binary)?)?;
    }
    Ok(())
}

pub fn marked_hooks_present(hooks_dir: &Path) -> bool {
    let owned = hooks_dir.join(OWNED_FILE_NAME);
    fs::read(&owned)
        .ok()
        .is_some_and(|bytes| bytes_have_marker(&bytes))
}

pub fn install_marked_grok_hooks(
    hooks_dir: &Path,
    command: &str,
) -> Result<(), GrokHookSettingsError> {
    if command_program_is_grok(command) {
        return Err(GrokHookSettingsError::GrokCommand);
    }
    fs::create_dir_all(hooks_dir).map_err(|error| io_error(hooks_dir, error))?;
    for path in json_files(hooks_dir)? {
        let bytes = fs::read(&path).map_err(|error| io_error(&path, error))?;
        if !bytes_have_marker(&bytes) {
            continue;
        }
        let updated = rewrite_marked(&path, &bytes, |value| {
            update_marked_commands(value, command);
        })?;
        atomic_write(&path, &updated)?;
    }
    let owned = hooks_dir.join(OWNED_FILE_NAME);
    if !owned.exists() {
        atomic_write(&owned, &owned_file_bytes(command)?)?;
    }
    Ok(())
}

pub fn uninstall_marked_grok_hooks(hooks_dir: &Path) -> Result<(), GrokHookSettingsError> {
    if !hooks_dir.exists() {
        return Ok(());
    }
    for path in json_files(hooks_dir)? {
        let bytes = fs::read(&path).map_err(|error| io_error(&path, error))?;
        if !bytes_have_marker(&bytes) {
            continue;
        }
        let updated = rewrite_marked(&path, &bytes, remove_marked_groups)?;
        if file_has_no_hooks(&updated) {
            fs::remove_file(&path).map_err(|error| io_error(&path, error))?;
        } else {
            atomic_write(&path, &updated)?;
        }
    }
    Ok(())
}

fn command_program_is_grok(command: &str) -> bool {
    let Some(program) = command.split_whitespace().next() else {
        return false;
    };
    Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        == Some("grok")
}

fn bytes_have_marker(bytes: &[u8]) -> bool {
    let needle = LIBRA_CAPTURE_MARKER.as_bytes();
    bytes.windows(needle.len()).any(|window| window == needle)
}

fn json_files(dir: &Path) -> Result<Vec<PathBuf>, GrokHookSettingsError> {
    let mut paths = Vec::new();
    let entries = fs::read_dir(dir).map_err(|error| io_error(dir, error))?;
    for entry in entries {
        let entry = entry.map_err(|error| io_error(dir, error))?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path).map_err(|error| io_error(&path, error))?;
        if meta.file_type().is_symlink() || !meta.file_type().is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn rewrite_marked(
    path: &Path,
    bytes: &[u8],
    edit: impl FnOnce(&mut Value),
) -> Result<Vec<u8>, GrokHookSettingsError> {
    let mut value: Value =
        serde_json::from_slice(bytes).map_err(|error| GrokHookSettingsError::Malformed {
            path: path.display().to_string(),
            detail: error.to_string(),
        })?;
    edit(&mut value);
    serde_json::to_vec_pretty(&value).map_err(|error| GrokHookSettingsError::Malformed {
        path: path.display().to_string(),
        detail: error.to_string(),
    })
}

fn update_marked_commands(value: &mut Value, command: &str) {
    let Some(groups_by_event) = value.get_mut("hooks").and_then(Value::as_object_mut) else {
        return;
    };
    for groups in groups_by_event.values_mut() {
        let Some(groups) = groups.as_array_mut() else {
            continue;
        };
        for group in groups {
            if !group_is_marked(group) {
                continue;
            }
            let Some(entries) = group.get_mut("hooks").and_then(Value::as_array_mut) else {
                continue;
            };
            for entry in entries {
                if entry.get("type").and_then(Value::as_str) == Some("command") {
                    entry["command"] = Value::String(command.to_string());
                }
            }
        }
    }
}

fn remove_marked_groups(value: &mut Value) {
    let Some(groups_by_event) = value.get_mut("hooks").and_then(Value::as_object_mut) else {
        return;
    };
    let events: Vec<String> = groups_by_event.keys().cloned().collect();
    for event in events {
        let Some(groups) = groups_by_event
            .get_mut(&event)
            .and_then(Value::as_array_mut)
        else {
            continue;
        };
        groups.retain(|group| !group_is_marked(group));
        if groups.is_empty() {
            groups_by_event.remove(&event);
        }
    }
}

fn group_is_marked(group: &Value) -> bool {
    group.get("statusMessage").and_then(Value::as_str) == Some(LIBRA_CAPTURE_MARKER)
}

fn file_has_no_hooks(bytes: &[u8]) -> bool {
    let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
        return false;
    };
    let Some(object) = value.as_object() else {
        return false;
    };
    let hooks_empty = object
        .get("hooks")
        .and_then(Value::as_object)
        .is_none_or(|hooks| hooks.is_empty());
    let extra = object.keys().any(|key| key != "hooks");
    hooks_empty && !extra
}

fn update_marked_commands_for_binary(value: &mut Value, binary: &str) {
    let Some(groups_by_event) = value.get_mut("hooks").and_then(Value::as_object_mut) else {
        return;
    };
    let events: Vec<String> = groups_by_event.keys().cloned().collect();
    for event in events {
        let command = grok_hook_command(binary, &event);
        let Some(groups) = groups_by_event
            .get_mut(&event)
            .and_then(Value::as_array_mut)
        else {
            continue;
        };
        for group in groups {
            if !group_is_marked(group) {
                continue;
            }
            let Some(entries) = group.get_mut("hooks").and_then(Value::as_array_mut) else {
                continue;
            };
            for entry in entries {
                if entry.get("type").and_then(Value::as_str) == Some("command") {
                    entry["command"] = Value::String(command.clone());
                }
            }
        }
    }
}

fn owned_file_bytes_for_binary(binary: &str) -> Result<Vec<u8>, GrokHookSettingsError> {
    let mut hooks = Map::new();
    for event in BOUNDARY_EVENTS {
        hooks.insert(
            (*event).to_string(),
            Value::Array(vec![Value::Object(marked_group(&grok_hook_command(
                binary, event,
            )))]),
        );
    }
    let mut root = Map::new();
    root.insert("hooks".to_string(), Value::Object(hooks));
    serde_json::to_vec_pretty(&Value::Object(root)).map_err(|error| {
        GrokHookSettingsError::Malformed {
            path: OWNED_FILE_NAME.to_string(),
            detail: error.to_string(),
        }
    })
}

fn owned_file_bytes(command: &str) -> Result<Vec<u8>, GrokHookSettingsError> {
    let mut hooks = Map::new();
    for event in BOUNDARY_EVENTS {
        hooks.insert(
            (*event).to_string(),
            Value::Array(vec![Value::Object(marked_group(command))]),
        );
    }
    let mut root = Map::new();
    root.insert("hooks".to_string(), Value::Object(hooks));
    serde_json::to_vec_pretty(&Value::Object(root)).map_err(|error| {
        GrokHookSettingsError::Malformed {
            path: OWNED_FILE_NAME.to_string(),
            detail: error.to_string(),
        }
    })
}

fn marked_group(command: &str) -> Map<String, Value> {
    let mut group = Map::new();
    group.insert(
        "statusMessage".to_string(),
        Value::String(LIBRA_CAPTURE_MARKER.to_string()),
    );
    group.insert(
        "hooks".to_string(),
        Value::Array(vec![serde_json::json!({
            "type": "command",
            "command": command,
            "timeout": 10,
        })]),
    );
    group
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), GrokHookSettingsError> {
    let tmp = path.with_extension("json.libra-tmp");
    {
        let mut file = File::create(&tmp).map_err(|error| io_error(&tmp, error))?;
        file.write_all(bytes)
            .map_err(|error| io_error(&tmp, error))?;
        file.write_all(b"\n")
            .map_err(|error| io_error(&tmp, error))?;
        file.sync_all().map_err(|error| io_error(&tmp, error))?;
    }
    fs::rename(&tmp, path).map_err(|error| io_error(path, error))?;
    Ok(())
}

fn io_error(path: &Path, error: std::io::Error) -> GrokHookSettingsError {
    GrokHookSettingsError::Io {
        path: path.display().to_string(),
        detail: error.to_string(),
    }
}
