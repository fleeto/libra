//! One admission policy for the Hermes producer and receiver.
//!
//! The verified worktree root comes from the caller. This module does not
//! look up a global profile, an event field, or a Hermes home.

use std::path::{Component, Path, PathBuf};

/// Worktree fact already checked by capture scope. `profile` is the profile
/// recorded for this session, not a value read from configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermesVerifiedWorktree {
    pub root: PathBuf,
    pub profile: String,
}

/// Header fields from the selected Hermes session. Message bodies are absent
/// on purpose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermesScopeCandidate {
    pub platform: Option<String>,
    pub created_source: Option<String>,
    pub user_id: Option<String>,
    pub cwd: Option<String>,
    pub recorded_profile: Option<String>,
}

/// Content-free admission result. Refusal text never includes cwd, profile,
/// or session text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HermesScopeDecision {
    Admit,
    Refuse(&'static str),
}

impl HermesScopeDecision {
    pub fn is_admit(self) -> bool {
        matches!(self, Self::Admit)
    }
}

/// Producer entry. Same policy as [`admit_receiver`].
pub fn admit_producer(
    verified: &HermesVerifiedWorktree,
    candidate: &HermesScopeCandidate,
) -> HermesScopeDecision {
    admit_hermes_scope(verified, candidate)
}

/// Receiver entry. Same policy as [`admit_producer`].
pub fn admit_receiver(
    verified: &HermesVerifiedWorktree,
    candidate: &HermesScopeCandidate,
) -> HermesScopeDecision {
    admit_hermes_scope(verified, candidate)
}

/// Admit only a local CLI session whose cwd is exactly the verified worktree
/// and whose recorded profile is the caller's profile.
pub fn admit_hermes_scope(
    verified: &HermesVerifiedWorktree,
    candidate: &HermesScopeCandidate,
) -> HermesScopeDecision {
    if !is_local_cli(candidate.platform.as_deref()) {
        return HermesScopeDecision::Refuse("hermes scope platform is not the local cli");
    }
    if !created_source_is_local_cli(candidate.created_source.as_deref()) {
        return HermesScopeDecision::Refuse("hermes scope created source is not the local cli");
    }
    if candidate
        .user_id
        .as_deref()
        .is_some_and(|user| !user.is_empty())
    {
        return HermesScopeDecision::Refuse("hermes scope user is not the local session");
    }
    let Some(cwd) = candidate.cwd.as_deref().filter(|cwd| !cwd.is_empty()) else {
        return HermesScopeDecision::Refuse("hermes scope is missing cwd");
    };
    if !cwd_equals_verified(Path::new(cwd), &verified.root) {
        return HermesScopeDecision::Refuse("hermes scope cwd is not the verified worktree");
    }
    if !profile_matches(candidate.recorded_profile.as_deref(), &verified.profile) {
        return HermesScopeDecision::Refuse(
            "hermes scope profile does not match the recorded session",
        );
    }
    HermesScopeDecision::Admit
}

fn is_local_cli(platform: Option<&str>) -> bool {
    platform.is_some_and(|value| value.eq_ignore_ascii_case("cli"))
}

fn created_source_is_local_cli(created: Option<&str>) -> bool {
    match created {
        None | Some("") => true,
        Some(value) => value.eq_ignore_ascii_case("cli"),
    }
}

fn profile_matches(recorded: Option<&str>, caller: &str) -> bool {
    if caller.is_empty() {
        return false;
    }
    match recorded {
        Some(name) if !name.is_empty() => name == caller,
        _ => caller == "default",
    }
}

fn cwd_equals_verified(cwd: &Path, root: &Path) -> bool {
    if !cwd.is_absolute() || !root.is_absolute() {
        return false;
    }
    if has_dot_component(cwd) || has_dot_component(root) {
        return false;
    }
    cwd == root
}

fn has_dot_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::ai::observed_agents::{
        AgentKind, agent_for, live_capture_matches_hook_installable, registration_for,
    };

    struct Case {
        platform: Option<&'static str>,
        created_source: Option<&'static str>,
        user_id: Option<&'static str>,
        cwd: CwdCase,
        recorded_profile: Option<&'static str>,
        caller_profile: &'static str,
        admit: bool,
    }

    enum CwdCase {
        Root,
        Child,
        Relative,
        Parent,
        Missing,
        Other,
    }

    fn matrix() -> Vec<Case> {
        vec![
            Case {
                platform: Some("cli"),
                created_source: None,
                user_id: None,
                cwd: CwdCase::Root,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: true,
            },
            Case {
                platform: Some("CLI"),
                created_source: Some("cli"),
                user_id: Some(""),
                cwd: CwdCase::Root,
                recorded_profile: None,
                caller_profile: "default",
                admit: true,
            },
            Case {
                platform: Some("cli"),
                created_source: Some(""),
                user_id: None,
                cwd: CwdCase::Root,
                recorded_profile: Some("work"),
                caller_profile: "work",
                admit: true,
            },
            Case {
                platform: Some("gateway"),
                created_source: None,
                user_id: None,
                cwd: CwdCase::Root,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: Some("telegram"),
                created_source: Some("cli"),
                user_id: None,
                cwd: CwdCase::Root,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: Some("discord"),
                created_source: None,
                user_id: None,
                cwd: CwdCase::Root,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: Some("matrix"),
                created_source: None,
                user_id: None,
                cwd: CwdCase::Root,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: Some("remote"),
                created_source: None,
                user_id: None,
                cwd: CwdCase::Root,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: None,
                created_source: None,
                user_id: None,
                cwd: CwdCase::Root,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: Some("cli"),
                created_source: Some("gateway"),
                user_id: None,
                cwd: CwdCase::Root,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: Some("cli"),
                created_source: None,
                user_id: Some("other-user"),
                cwd: CwdCase::Root,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: Some("cli"),
                created_source: None,
                user_id: None,
                cwd: CwdCase::Missing,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: Some("cli"),
                created_source: None,
                user_id: None,
                cwd: CwdCase::Relative,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: Some("cli"),
                created_source: None,
                user_id: None,
                cwd: CwdCase::Child,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: Some("cli"),
                created_source: None,
                user_id: None,
                cwd: CwdCase::Parent,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: Some("cli"),
                created_source: None,
                user_id: None,
                cwd: CwdCase::Other,
                recorded_profile: Some("default"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: Some("cli"),
                created_source: None,
                user_id: None,
                cwd: CwdCase::Root,
                recorded_profile: Some("work"),
                caller_profile: "default",
                admit: false,
            },
            Case {
                platform: Some("cli"),
                created_source: None,
                user_id: None,
                cwd: CwdCase::Root,
                recorded_profile: None,
                caller_profile: "work",
                admit: false,
            },
            Case {
                platform: Some("cli"),
                created_source: None,
                user_id: None,
                cwd: CwdCase::Root,
                recorded_profile: Some("default"),
                caller_profile: "",
                admit: false,
            },
        ]
    }

    fn materialize(case: &Case, root: &Path) -> (HermesVerifiedWorktree, HermesScopeCandidate) {
        let cwd = match case.cwd {
            CwdCase::Root => Some(root.display().to_string()),
            CwdCase::Child => Some(root.join("nested").display().to_string()),
            CwdCase::Relative => Some("nested".to_string()),
            CwdCase::Parent => Some(root.join("..").display().to_string()),
            CwdCase::Missing => None,
            CwdCase::Other => {
                let parent = root.parent().unwrap_or(root);
                Some(
                    parent
                        .join("libra-hermes-other-worktree")
                        .display()
                        .to_string(),
                )
            }
        };
        (
            HermesVerifiedWorktree {
                root: root.to_path_buf(),
                profile: case.caller_profile.to_string(),
            },
            HermesScopeCandidate {
                platform: case.platform.map(str::to_string),
                created_source: case.created_source.map(str::to_string),
                user_id: case.user_id.map(str::to_string),
                cwd,
                recorded_profile: case.recorded_profile.map(str::to_string),
            },
        )
    }

    #[test]
    fn hermes_scope_admission_matrix_agrees() {
        let root = std::env::temp_dir().join("libra-hermes-scope-root");
        assert!(root.is_absolute());
        for case in matrix() {
            let (verified, candidate) = materialize(&case, &root);
            let producer = admit_producer(&verified, &candidate);
            let receiver = admit_receiver(&verified, &candidate);
            assert_eq!(producer, receiver);
            assert_eq!(
                producer.is_admit(),
                case.admit,
                "{producer:?} {candidate:?}"
            );
            let rendered = format!("{producer:?} {receiver:?}");
            assert!(!rendered.contains("other-user"));
            if let Some(cwd) = candidate.cwd.as_deref() {
                assert!(!rendered.contains(cwd));
            }
        }
        for case in matrix() {
            let (verified, candidate) = materialize(&case, &root);
            let payload = serde_json::json!({
                "platform": candidate.platform,
                "created_source": candidate.created_source,
                "user_id": candidate.user_id,
                "cwd": candidate.cwd,
                "profile_name": candidate.recorded_profile,
                "verified_root": verified.root.display().to_string(),
                "caller_profile": verified.profile,
                "event_cwd": root.join("from-event").display().to_string(),
                "profile": "from-event",
            });
            let output = std::process::Command::new("python3")
                .arg(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/internal/ai/hooks/providers/hermes/libra_observer.py"
                ))
                .arg("--admit")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("spawn observer");
            let mut child = output;
            {
                use std::io::Write;
                child
                    .stdin
                    .take()
                    .expect("stdin")
                    .write_all(payload.to_string().as_bytes())
                    .expect("write payload");
            }
            let finished = child.wait_with_output().expect("observer");
            assert!(
                finished.status.success(),
                "{}",
                String::from_utf8_lossy(&finished.stderr)
            );
            let text = String::from_utf8(finished.stdout).expect("utf8");
            assert_eq!(
                text.trim(),
                if case.admit { "admit" } else { "refuse" },
                "{payload}"
            );
        }
    }

    #[test]
    fn hermes_scope_admission_parser_stops_before_the_body() {
        use serde_json::json;
        let root = std::env::temp_dir().join("libra-hermes-scope-parser");
        let verified = HermesVerifiedWorktree {
            root: root.clone(),
            profile: "default".to_string(),
        };
        let refused = json!({
            "source": "cli",
            "messages": "secret-thinking-body",
            "event_cwd": root.display().to_string(),
            "profile": "default",
        });
        let decision = crate::internal::ai::hooks::providers::hermes::parse_hermes_scope_value(
            &refused, &verified,
        );
        assert_eq!(
            decision,
            HermesScopeDecision::Refuse("hermes scope is missing cwd")
        );
        assert!(!format!("{decision:?}").contains("secret-thinking-body"));

        let admitted = json!({
            "source": "cli",
            "cwd": root.display().to_string(),
            "profile_name": "default",
            "messages": [{"reasoning": "secret-thinking-body"}],
        });
        let decision = crate::internal::ai::hooks::providers::hermes::parse_hermes_scope_value(
            &admitted, &verified,
        );
        assert_eq!(decision, HermesScopeDecision::Admit);
        assert!(!format!("{decision:?}").contains("secret-thinking-body"));
    }

    #[tokio::test]
    async fn hermes_scope_admission_refusal_does_not_write_a_capture_row() {
        let repo = tempfile::tempdir().expect("repo");
        crate::utils::test::setup_with_new_libra_in(repo.path()).await;
        let before = capture_rows(repo.path());
        let root = repo.path().canonicalize().expect("canonical repo");
        let verified = HermesVerifiedWorktree {
            root: root.clone(),
            profile: "default".to_string(),
        };
        let refused = HermesScopeCandidate {
            platform: Some("gateway".to_string()),
            created_source: None,
            user_id: None,
            cwd: None,
            recorded_profile: Some("default".to_string()),
        };
        assert!(!admit_receiver(&verified, &refused).is_admit());
        let admitted = HermesScopeCandidate {
            platform: Some("cli".to_string()),
            created_source: None,
            user_id: None,
            cwd: Some(root.display().to_string()),
            recorded_profile: Some("default".to_string()),
        };
        assert!(admit_receiver(&verified, &admitted).is_admit());
        assert_eq!(capture_rows(repo.path()), before);
        assert_eq!(before, 0);
    }

    fn capture_rows(repo: &Path) -> i64 {
        let db = repo.join(".libra").join("libra.db");
        let output = std::process::Command::new("python3")
            .arg("-c")
            .arg("import sqlite3,sys; print(sqlite3.connect(sys.argv[1]).execute('select count(*) from agent_session').fetchone()[0])")
            .arg(&db)
            .output()
            .expect("count agent_session");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("count utf8")
            .trim()
            .parse()
            .expect("count")
    }

    #[test]
    fn hermes_scope_admission_does_not_promote_the_roster() {
        let row = registration_for(AgentKind::Hermes);
        assert!(!row.supported);
        assert!(!row.hook_installable);
        assert!(!row.transcript_readable);
        assert!(!row.launchable_review);
        assert!(!row.launchable_investigate);
        assert!(agent_for(AgentKind::Hermes).as_hooks().is_none());
        assert!(live_capture_matches_hook_installable(AgentKind::Hermes));
    }
}
