//! CC-01: the Claude hook registry matches the frozen CC-00 name matrix.
//!
//! These tests call the shipped registry and the provider parser. They do not
//! treat a count of 33 as the contract.

use libra::internal::ai::hooks::{
    SessionHookEnvelope, claude_provider,
    providers::claude::{
        ClaudeEventClass, classify_claude_fields, control_sensitive_names, deprecated_alias_names,
        event_class, known_event_names, safely_installable_names,
    },
};
use serde_json::{Map, Value};

const INSTALLABLE: &[&str] = &[
    "SessionStart",
    "Setup",
    "UserPromptSubmit",
    "UserPromptExpansion",
    "PreToolUse",
    "PermissionRequest",
    "PermissionDenied",
    "PostToolUse",
    "PostToolUseFailure",
    "PostToolBatch",
    "Notification",
    "MessageDisplay",
    "SubagentStart",
    "SubagentStop",
    "TaskCreated",
    "TaskCompleted",
    "Stop",
    "StopFailure",
    "TeammateIdle",
    "InstructionsLoaded",
    "ConfigChange",
    "CwdChanged",
    "DirectoryAdded",
    "FileChanged",
    "PreCompact",
    "PostCompact",
    "PostModelSwitch",
    "Elicitation",
    "ElicitationResult",
    "SessionEnd",
];

const HOLD: &[&str] = &["WorktreeCreate", "WorktreeRemove", "PreModelSwitch"];
const DEPRECATED: &[&str] = &["SessionStop", "ModelUpdate", "Compaction"];

#[test]
fn registry_matches_the_frozen_claude_event_matrix() {
    let installable = safely_installable_names();
    let hold = control_sensitive_names();
    let deprecated = deprecated_alias_names();
    let known = known_event_names();

    assert_eq!(installable, INSTALLABLE);
    assert_eq!(hold, HOLD);
    assert_eq!(deprecated, DEPRECATED);
    assert_eq!(
        known.len(),
        installable.len() + hold.len() + deprecated.len()
    );
    assert_ne!(
        installable.len(),
        known.len(),
        "installable coverage is a subset, not the whole known table"
    );

    for name in deprecated.iter().chain(hold.iter()) {
        assert!(
            !installable.iter().any(|installable| installable == name),
            "{name} must stay out of the current installable set"
        );
    }
    for name in &hold {
        assert!(
            !claude_provider().recognizes_event(name),
            "{name} is control-sensitive and must not enter the current ingress"
        );
    }
    for name in &deprecated {
        assert!(
            claude_provider().recognizes_event(name),
            "deprecated alias {name} stays recognizable for old forwarders"
        );
    }
}

#[test]
fn registry_unknown_event_and_field_stay_partial_without_the_payload() {
    let canary = "CC01-SECRET-CANARY-VALUE";
    let unknown = classify_claude_fields("NotAClaudeEvent", &[canary]);
    assert!(unknown.partial);
    assert_eq!(unknown.diagnostic, "unknown Claude hook event");
    assert!(!unknown.diagnostic.contains(canary));

    let mut extra = Map::new();
    extra.insert(canary.to_string(), Value::String(canary.to_string()));
    extra.insert("prompt".to_string(), Value::String("hello".to_string()));
    let envelope = SessionHookEnvelope {
        hook_event_name: "SessionStart".to_string(),
        session_id: "s1".to_string(),
        cwd: "/tmp/cc01".to_string(),
        transcript_path: None,
        extra,
    };
    let event = claude_provider()
        .parse_hook_event("SessionStart", &envelope)
        .expect("known SessionStart still parses");
    let rendered = format!("{event:?}");
    assert!(!rendered.contains(canary));
    assert_eq!(event.prompt.as_deref(), Some("hello"));

    let fields = classify_claude_fields("SessionStart", &[canary, "prompt", "source"]);
    assert!(fields.partial);
    assert_eq!(fields.diagnostic, "unknown Claude hook field");
    assert!(!fields.diagnostic.contains(canary));

    let parse_error = claude_provider()
        .parse_hook_event(canary, &envelope)
        .expect_err("unknown event is not ingress");
    let parse_error = format!("{parse_error:#}");
    assert!(!parse_error.contains(canary));
    assert!(parse_error.contains("unknown Claude Code hook event"));
}

#[test]
fn registry_deprecated_alias_parses_and_is_excluded_from_installable() {
    let envelope = SessionHookEnvelope {
        hook_event_name: "SessionStop".to_string(),
        session_id: "s1".to_string(),
        cwd: "/tmp/cc01".to_string(),
        transcript_path: None,
        extra: Map::new(),
    };
    let event = claude_provider()
        .parse_hook_event("SessionStop", &envelope)
        .expect("deprecated alias still parses");
    assert_eq!(
        event.kind,
        libra::internal::ai::hooks::LifecycleEventKind::TurnEnd
    );
    assert!(deprecated_alias_names().contains(&"SessionStop"));
    assert!(!safely_installable_names().contains(&"SessionStop"));
    assert_eq!(
        event_class("SessionStop"),
        Some(ClaudeEventClass::DeprecatedAlias)
    );
}
