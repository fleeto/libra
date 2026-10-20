//! Single Claude Code hook-event registry.
//!
//! CC-00 froze the names, the safely installable set, the three
//! control-sensitive holds, and the deprecated aliases. This module is the
//! only taxonomy. The parser and `recognizes_event` consult it; they do not
//! keep a second name list. The integer size of the table is not a contract.

use super::super::super::lifecycle::LifecycleEventKind;

/// How a frozen name relates to installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeEventClass {
    /// Empty stdout and exit 0 do not take over the host action.
    SafelyInstallable,
    /// Known, but installing would take over creation, cleanup, or model switch.
    ControlSensitive,
    /// Recognized old name. Not part of the current installable set.
    DeprecatedAlias,
}

/// Result of comparing extra keys with the registry allowlist.
///
/// `diagnostic` is a static sentence. It never includes the event name or a
/// field value, so a partial result cannot echo a payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaudeFieldReport {
    pub partial: bool,
    pub diagnostic: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ClaudeEventSpec {
    pub name: &'static str,
    pub class: ClaudeEventClass,
    /// The current verb parser accepts this name. Deferred names stay
    /// unrecognized by the dispatcher until a later card wires them.
    pub on_current_ingress: bool,
    pub kind: Option<LifecycleEventKind>,
    known_fields: &'static [&'static str],
}

const fn ev(
    name: &'static str,
    class: ClaudeEventClass,
    on_current_ingress: bool,
    kind: Option<LifecycleEventKind>,
    known_fields: &'static [&'static str],
) -> ClaudeEventSpec {
    ClaudeEventSpec {
        name,
        class,
        on_current_ingress,
        kind,
        known_fields,
    }
}

const COMMON_FIELDS: &[&str] = &[
    "prompt_id",
    "scratchpad_dir",
    "permission_mode",
    "effort",
    "agent_id",
    "agent_type",
    "thinking",
];

const NONE: &[&str] = &[];
const PROMPT: &[&str] = &["prompt"];
const SOURCE_MODEL: &[&str] = &[
    "source",
    "model",
    "session_title",
    "seconds_since_last_response",
    "context_tokens",
    "prompt_cache_likely_expired",
    "estimated_cache_write_usd",
];
const TOOL: &[&str] = &[
    "tool_name",
    "tool_input",
    "tool_use_id",
    "tool_response",
    "duration_ms",
    "mcp_server",
    "is_interrupt",
    "error",
];
const MODEL_SWITCH: &[&str] = &[
    "from_model",
    "to_model",
    "requested_model",
    "source",
    "context_tokens",
    "prompt_cache_warm",
    "cache_ttl",
    "estimated_cache_write_usd",
    "pricing",
];

use ClaudeEventClass::{ControlSensitive, DeprecatedAlias, SafelyInstallable};
use LifecycleEventKind::{
    Compaction, CompactionCompleted, ModelUpdate, PermissionRequest, SessionEnd, SessionStart,
    SubagentEnd, SubagentStart, ToolUse, TurnEnd, TurnStart,
};

/// Frozen CC-00 names. Order is the registry order, not a cardinality contract.
pub static CLAUDE_EVENT_REGISTRY: &[ClaudeEventSpec] = &[
    ev(
        "SessionStart",
        SafelyInstallable,
        true,
        Some(SessionStart),
        SOURCE_MODEL,
    ),
    ev(
        "Setup",
        SafelyInstallable,
        true,
        Some(SessionStart),
        &["trigger"],
    ),
    ev(
        "UserPromptSubmit",
        SafelyInstallable,
        true,
        Some(TurnStart),
        PROMPT,
    ),
    ev(
        "UserPromptExpansion",
        SafelyInstallable,
        true,
        Some(TurnStart),
        &[
            "expansion_type",
            "command_name",
            "command_args",
            "command_source",
            "prompt",
        ],
    ),
    ev("PreToolUse", SafelyInstallable, true, Some(ToolUse), TOOL),
    ev(
        "PermissionRequest",
        SafelyInstallable,
        true,
        Some(PermissionRequest),
        &[
            "tool_name",
            "tool_input",
            "mcp_server",
            "permission_suggestions",
        ],
    ),
    ev(
        "PermissionDenied",
        SafelyInstallable,
        true,
        Some(ToolUse),
        &[
            "tool_name",
            "tool_input",
            "tool_use_id",
            "reason",
            "mcp_server",
        ],
    ),
    ev("PostToolUse", SafelyInstallable, true, Some(ToolUse), TOOL),
    ev(
        "PostToolUseFailure",
        SafelyInstallable,
        true,
        Some(ToolUse),
        TOOL,
    ),
    ev(
        "PostToolBatch",
        SafelyInstallable,
        true,
        Some(ToolUse),
        &["tool_calls"],
    ),
    ev(
        "Notification",
        SafelyInstallable,
        true,
        Some(ToolUse),
        &["message", "title", "notification_type"],
    ),
    ev(
        "MessageDisplay",
        SafelyInstallable,
        true,
        Some(ToolUse),
        &["turn_id", "message_id", "index", "final", "delta"],
    ),
    ev(
        "SubagentStart",
        SafelyInstallable,
        true,
        Some(SubagentStart),
        &["agent_id", "agent_type"],
    ),
    ev(
        "SubagentStop",
        SafelyInstallable,
        true,
        Some(SubagentEnd),
        &[
            "stop_hook_active",
            "agent_id",
            "agent_type",
            "agent_transcript_path",
            "last_assistant_message",
        ],
    ),
    ev(
        "TaskCreated",
        SafelyInstallable,
        true,
        Some(ToolUse),
        &[
            "task_id",
            "task_subject",
            "task_description",
            "teammate_name",
            "team_name",
        ],
    ),
    ev(
        "TaskCompleted",
        SafelyInstallable,
        true,
        Some(ToolUse),
        &[
            "task_id",
            "task_subject",
            "task_description",
            "teammate_name",
            "team_name",
        ],
    ),
    ev(
        "Stop",
        SafelyInstallable,
        true,
        Some(TurnEnd),
        &[
            "stop_hook_active",
            "last_assistant_message",
            "background_tasks",
            "session_crons",
        ],
    ),
    ev(
        "StopFailure",
        SafelyInstallable,
        true,
        Some(ToolUse),
        &["error", "error_details", "last_assistant_message"],
    ),
    ev(
        "TeammateIdle",
        SafelyInstallable,
        true,
        Some(ToolUse),
        &["teammate_name", "team_name"],
    ),
    ev(
        "InstructionsLoaded",
        SafelyInstallable,
        true,
        Some(SessionStart),
        &[
            "file_path",
            "memory_type",
            "load_reason",
            "globs",
            "trigger_file_path",
            "parent_file_path",
        ],
    ),
    ev(
        "ConfigChange",
        SafelyInstallable,
        true,
        Some(SessionStart),
        &["source", "file_path"],
    ),
    ev(
        "CwdChanged",
        SafelyInstallable,
        true,
        Some(SessionStart),
        &["old_cwd", "new_cwd"],
    ),
    ev(
        "DirectoryAdded",
        SafelyInstallable,
        true,
        Some(SessionStart),
        &["directory", "source"],
    ),
    ev(
        "FileChanged",
        SafelyInstallable,
        true,
        Some(SessionStart),
        &["file_path", "event"],
    ),
    ev("WorktreeCreate", ControlSensitive, false, None, &["name"]),
    ev(
        "WorktreeRemove",
        ControlSensitive,
        false,
        None,
        &["worktree_path"],
    ),
    ev(
        "PreCompact",
        SafelyInstallable,
        true,
        Some(Compaction),
        &["trigger", "custom_instructions"],
    ),
    ev(
        "PostCompact",
        SafelyInstallable,
        true,
        Some(CompactionCompleted),
        &["trigger", "compact_summary"],
    ),
    ev(
        "PreModelSwitch",
        ControlSensitive,
        false,
        None,
        MODEL_SWITCH,
    ),
    ev(
        "PostModelSwitch",
        SafelyInstallable,
        true,
        Some(ModelUpdate),
        MODEL_SWITCH,
    ),
    ev(
        "Elicitation",
        SafelyInstallable,
        true,
        Some(ToolUse),
        &[
            "mcp_server_name",
            "message",
            "mode",
            "url",
            "elicitation_id",
            "requested_schema",
        ],
    ),
    ev(
        "ElicitationResult",
        SafelyInstallable,
        true,
        Some(ToolUse),
        &[
            "mcp_server_name",
            "action",
            "mode",
            "elicitation_id",
            "content",
        ],
    ),
    ev(
        "SessionEnd",
        SafelyInstallable,
        true,
        Some(SessionEnd),
        &["reason"],
    ),
    // Deprecated aliases. The current parser still accepts them so old
    // forwarders keep working. They are not installable.
    ev("SessionStop", DeprecatedAlias, true, Some(TurnEnd), NONE),
    ev(
        "ModelUpdate",
        DeprecatedAlias,
        true,
        Some(ModelUpdate),
        &["model"],
    ),
    ev("Compaction", DeprecatedAlias, true, Some(Compaction), NONE),
];

pub fn spec(name: &str) -> Option<&'static ClaudeEventSpec> {
    CLAUDE_EVENT_REGISTRY.iter().find(|spec| spec.name == name)
}

pub fn event_class(name: &str) -> Option<ClaudeEventClass> {
    spec(name).map(|spec| spec.class)
}

pub fn known_event_names() -> Vec<&'static str> {
    CLAUDE_EVENT_REGISTRY.iter().map(|spec| spec.name).collect()
}

pub fn safely_installable_names() -> Vec<&'static str> {
    names_in_class(ClaudeEventClass::SafelyInstallable)
}

pub fn control_sensitive_names() -> Vec<&'static str> {
    names_in_class(ClaudeEventClass::ControlSensitive)
}

pub fn deprecated_alias_names() -> Vec<&'static str> {
    names_in_class(ClaudeEventClass::DeprecatedAlias)
}

pub fn on_current_ingress(name: &str) -> bool {
    spec(name).is_some_and(|spec| spec.on_current_ingress)
}

pub use crate::internal::ai::capture::ingress::{
    MESSAGE_DISPLAY_COALESCE_LIMIT, admit_message_display, reset_message_display_counts_for_test,
};

/// Classify extra keys. Unknown events and unknown keys are partial.
/// The report never copies `keys` or their values into `diagnostic`.
pub fn classify_claude_fields(event_name: &str, keys: &[&str]) -> ClaudeFieldReport {
    let Some(spec) = spec(event_name) else {
        return ClaudeFieldReport {
            partial: true,
            diagnostic: "unknown Claude hook event",
        };
    };
    let unknown_key = keys.iter().any(|key| !field_allowed(spec, key));
    if unknown_key {
        ClaudeFieldReport {
            partial: true,
            diagnostic: "unknown Claude hook field",
        }
    } else {
        ClaudeFieldReport {
            partial: false,
            diagnostic: "claude hook fields are known",
        }
    }
}

fn names_in_class(class: ClaudeEventClass) -> Vec<&'static str> {
    CLAUDE_EVENT_REGISTRY
        .iter()
        .filter(|spec| spec.class == class)
        .map(|spec| spec.name)
        .collect()
}

fn field_allowed(spec: &ClaudeEventSpec, key: &str) -> bool {
    COMMON_FIELDS.contains(&key) || spec.known_fields.contains(&key)
}
