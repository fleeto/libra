//! RG-04 contract tests: readable reasoning projection and cross-provider
//! synthetic conformance fixtures (plan-20260904.md `### Task RG-04`).
//!
//! The fixtures under `tests/fixtures/agent_transcripts/reasoning/` are
//! entirely synthetic (no real provider content, per DEP-RG-01). Each fixture
//! pins one reasoning availability decision; the expectation table is embedded
//! in this file and diffed against the actual classification outcome.

use libra::internal::ai::observed_agents::reasoning::{
    ReasoningAvailability, ReasoningProvider, UnrecognizedReasoningField,
    classify_unrecognized_field,
};

const FIXTURE_DIR: &str = "tests/fixtures/agent_transcripts/reasoning";

fn read_fixture(name: &str) -> serde_json::Value {
    let path = format!("{FIXTURE_DIR}/{name}");
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path}: {error}"));
    serde_json::from_str(&raw).unwrap_or_else(|error| panic!("parse {path}: {error}"))
}

fn provider_tag(value: serde_json::Value) -> ReasoningProvider {
    match value.as_str() {
        Some("claude_code") => ReasoningProvider::ClaudeCode,
        Some("codex") => ReasoningProvider::Codex,
        Some("opencode") => ReasoningProvider::OpenCode,
        other => panic!("unknown provider tag {other:?}"),
    }
}

/// COV2-01 keeps the old normalized-turn wire shape and public reader
/// envelopes independent of the new private coverage-claim version. The CLI
/// cases use a synthetic Codex catalog row and do not start OpenCode or any
/// provider session.
#[tokio::test]
async fn cov2_turn_wire_and_public_envelope_are_stable() {
    use std::process::Command;

    use libra::internal::ai::observed_agents::coverage::NormalizedTurn;
    use sea_orm::{ConnectionTrait, Database};

    let wire = r#"{"logical_turn_key":"wire-turn","ordinal":0,"completeness":"complete","started_at":null,"ended_at":null,"records":[{"kind":"user","text":"hi"}]}"#;
    let turn: NormalizedTurn = serde_json::from_str(wire).expect("legacy turn wire decodes");
    assert_eq!(
        serde_json::to_string(&turn).expect("legacy turn wire encodes"),
        wire
    );

    let temp = tempfile::tempdir().expect("create isolated repository root");
    let repo = temp.path().join("repo");
    let home = temp.path().join("home");
    std::fs::create_dir_all(&repo).expect("create repository directory");
    std::fs::create_dir_all(&home).expect("create isolated home directory");
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_libra"))
            .current_dir(&repo)
            .env("HOME", &home)
            .env("LIBRA_TEST_HOME", &home)
            .env_remove("CODEX_HOME")
            .args(args)
            .output()
            .expect("run isolated Libra CLI")
    };

    let init = run(&["init"]);
    assert!(
        init.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );
    let list_v1 = run(&["agent", "list", "--json"]);
    assert!(
        list_v1.status.success(),
        "default list failed: {}",
        String::from_utf8_lossy(&list_v1.stderr)
    );
    let list_v1: serde_json::Value =
        serde_json::from_slice(&list_v1.stdout).expect("default list JSON");
    assert_eq!(list_v1["data"]["schema_version"], 1);

    let list_v2 = run(&["agent", "list", "--schema-version", "2", "--json"]);
    assert!(
        list_v2.status.success(),
        "explicit list v2 failed: {}",
        String::from_utf8_lossy(&list_v2.stderr)
    );
    let list_v2: serde_json::Value =
        serde_json::from_slice(&list_v2.stdout).expect("explicit list v2 JSON");
    assert_eq!(list_v2["data"]["schema_version"], 2);

    let database = Database::connect(format!(
        "sqlite://{}?mode=rwc",
        repo.join(".libra/libra.db").display()
    ))
    .await
    .expect("open isolated catalog");
    database
        .execute_unprepared(
            r#"INSERT INTO agent_session (
                session_id, agent_kind, provider_session_id, state, working_dir,
                metadata_json, redaction_report, started_at, last_event_at, schema_version
            ) VALUES (
                'codex__cov2-wire', 'codex', 'cov2-wire', 'stopped', '/synthetic',
                '{}', '{}', 1, 1, 1
            )"#,
        )
        .await
        .expect("seed synthetic catalog row");
    let graph = run(&["--json", "agent", "graph", "codex__cov2-wire"]);
    assert!(
        graph.status.success(),
        "graph failed: {}",
        String::from_utf8_lossy(&graph.stderr)
    );
    let graph: serde_json::Value = serde_json::from_slice(&graph.stdout).expect("graph JSON");
    assert_eq!(graph["data"]["schema_version"], 1);
}

/// AC 5: the existing coverage canonical digest is byte-for-byte unchanged by
/// the new Reasoning variant (regression pin on the coverage-v1.md §5
/// vector 1; the byte-level golden lives in coverage.rs golden_vector_1).
#[test]
fn existing_fixture_digest_unchanged() {
    use libra::internal::ai::observed_agents::coverage::{
        SemanticRecord, canonical_turn_bytes, coverage_digest_hex,
    };
    let records = vec![
        SemanticRecord::User {
            text: "hi".to_string(),
        },
        SemanticRecord::Assistant {
            text: "hello".to_string(),
        },
    ];
    assert_eq!(
        String::from_utf8_lossy(&canonical_turn_bytes(&records)),
        r#"[{"role":"user","text":"hi"},{"role":"assistant","text":"hello"}]"#
    );
    assert_eq!(
        coverage_digest_hex(&records),
        "df991cd9a1ac5c12c32b8cdf0254c3dfbbf26485b505a5afc83a90d1128ebc54"
    );
}

/// Conformance fixture conformance: each synthetic fixture drives one
/// availability decision; the actual outcome must equal the expectation
/// table below (diff must be empty).
#[test]
fn conformance_fixtures_match_expectation_table() {
    let visible = read_fixture("provider_visible.json");
    let not_present = read_fixture("not_present.json");
    let opencode = read_fixture("opencode.json");
    let generic_encrypted = read_fixture("generic_encrypted.json");

    // Expectation table (embedded golden; diff must be empty). The
    // classification INPUT is derived from the fixture record SHAPE, not the
    // filename — a fixture whose shape contradicts its expectation fails.
    let expectations: [(&str, ReasoningAvailability); 4] = [
        (
            "provider_visible.json",
            ReasoningAvailability::ProviderVisible,
        ),
        ("not_present.json", ReasoningAvailability::NotPresent),
        ("opencode.json", ReasoningAvailability::UnsupportedShape),
        ("generic_encrypted.json", ReasoningAvailability::NotPresent),
    ];

    for (name, expected) in expectations {
        let fixture = match name {
            "provider_visible.json" => &visible,
            "not_present.json" => &not_present,
            "opencode.json" => &opencode,
            _ => &generic_encrypted,
        };
        let record_json = &fixture["records"][0];
        let provider = provider_tag(record_json["provider"].clone());
        let actual = if record_json.get("reasoning_text").is_some() {
            // Shape: a recognized reasoning_text member on a supported
            // provider → ProviderVisible. The classification and projection
            // behavior itself is exercised by the crate-internal tests
            // (coverage.rs rg04_*), because external callers cannot mint the
            // gated text type by design; here we pin the fixture shape and
            // the expected availability (data-integrity diff).
            assert!(
                name == "provider_visible.json",
                "only the provider_visible fixture may carry reasoning_text"
            );
            assert_eq!(
                record_json["source_kind"], "reasoning",
                "provider_visible fixture must declare source_kind reasoning"
            );
            ReasoningAvailability::ProviderVisible
        } else if record_json.get("state").is_some() {
            // Shape: an OpenCode record carrying the open `state` member —
            // never proof of ciphertext → ReasoningLike (UnsupportedShape).
            assert!(
                name == "opencode.json",
                "only the opencode fixture may carry the open state member"
            );
            classify_unrecognized_field(
                provider,
                UnrecognizedReasoningField::ReasoningLike,
                record_json.to_string().as_bytes(),
            )
            .availability()
        } else if record_json.get("opaque_like_blob").is_some() {
            // Shape: an undeclared high-entropy blob member — entropy is
            // never evidence of ciphertext → NotPresent.
            assert!(
                name == "generic_encrypted.json",
                "only the generic_encrypted fixture may carry an undeclared blob"
            );
            classify_unrecognized_field(
                provider,
                UnrecognizedReasoningField::Other,
                record_json.to_string().as_bytes(),
            )
            .availability()
        } else {
            // Shape: no reasoning member at all → NotPresent.
            assert!(
                name == "not_present.json",
                "only the not_present fixture may lack every reasoning member"
            );
            classify_unrecognized_field(
                provider,
                UnrecognizedReasoningField::Other,
                record_json.to_string().as_bytes(),
            )
            .availability()
        };
        assert_eq!(
            actual, expected,
            "fixture {name} availability diverges from the expectation table"
        );
    }
}

/// AC 7 (warning half): unsupported-shape classification carries a
/// payload-free structured warning; the input bytes never appear in it
/// (Claude re-review P2: assert payload absence, not just existence).
#[test]
fn unsupported_shape_warning_is_payload_free() {
    let payload = "secret-payload-CANARY-7f2a1";
    let record = classify_unrecognized_field(
        ReasoningProvider::OpenCode,
        UnrecognizedReasoningField::ReasoningLike,
        payload.as_bytes(),
    );
    assert_eq!(
        record.availability(),
        ReasoningAvailability::UnsupportedShape
    );
    let warning = record.warning().expect("structured warning present");
    let serialized = serde_json::to_string(&warning).expect("warning serializes");
    assert!(
        !serialized.contains("CANARY") && !serialized.contains("secret-payload"),
        "the warning must be payload-free, got: {serialized}"
    );
}

#[test]
fn opencode_visible_reasoning_stays_out_of_coverage_v1() {
    use libra::internal::ai::observed_agents::coverage::{
        Completeness, SemanticRecord, normalize_opencode_export, safe_turn_projection,
    };

    let source = br#"{"info":{"id":"ses_og25"},"messages":[{"id":"u1","type":"user","text":"review"},{"id":"a1","type":"assistant","content":[{"id":"r1","type":"reasoning","text":"thought sk-ant-EXAMPLE0000000000000000"}]},{"id":"i1","type":"idle","outcome":"succeeded"}]}"#;
    let turns = normalize_opencode_export(source);
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].completeness, Completeness::Incomplete);
    assert!(
        !turns[0]
            .records
            .iter()
            .any(|record| matches!(record, SemanticRecord::Reasoning { .. }))
    );
    let projection = safe_turn_projection("opencode", &turns[0]).to_string();
    assert!(!projection.contains("\"type\":\"reasoning\""));
    assert!(!projection.contains("sk-ant-EXAMPLE0000000000000000"));
}

#[test]
fn opencode_unknown_reasoning_shape_partial_warning() {
    use libra::internal::ai::observed_agents::coverage::{
        Completeness, SemanticRecord, normalize_opencode_export, safe_turn_projection,
    };

    let canary = "CIPHER_UNKNOWN_OG25";
    let source = format!(
        r#"{{"info":{{"id":"ses_og25"}},"messages":[{{"id":"u1","type":"user","text":"review"}},{{"id":"a1","type":"assistant","content":[{{"id":"r1","type":"reasoning","text":"visible","state":{{"unknown":"{canary}"}}}}]}}]}}"#
    );
    let turns = normalize_opencode_export(source.as_bytes());
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].completeness, Completeness::Incomplete);
    assert!(
        !turns[0]
            .records
            .iter()
            .any(|record| matches!(record, SemanticRecord::Reasoning { .. }))
    );
    let projection = safe_turn_projection("opencode", &turns[0]).to_string();
    assert!(!projection.contains(canary));
    let warning = classify_unrecognized_field(
        ReasoningProvider::OpenCode,
        UnrecognizedReasoningField::ReasoningLike,
        canary.as_bytes(),
    )
    .warning()
    .expect("unsupported shape warning");
    let warning = serde_json::to_string(&warning).expect("payload-free warning serializes");
    assert!(!warning.contains(canary));
}
