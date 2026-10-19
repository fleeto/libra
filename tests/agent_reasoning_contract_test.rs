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

/// The shared canonical contract is pinned independently of either writer.
/// The two real temp-DB writer tests in lib use this same literal SHA; this
/// external test exercises only the bounded memory codec, never a Session.
#[test]
fn cov2_production_typed_v2_distinct_from_rg04_legacy() {
    use libra::internal::ai::{
        capture::snapshot::{CaptureSnapshotPolicy, CaptureSnapshotService},
        observed_agents::{canonical_turn_bytes, normalize_opencode_export, redact_turns},
    };
    use sha2::{Digest, Sha256};

    const RAW_SECRET: &str = "sk-ant-EXAMPLE0000000000000000";
    const V2_CANONICAL: &str = r#"[{"role":"user","text":"hi"},{"provider":"opencode","role":"reasoning","source_kind":"reasoning","text":"before <REDACTED:anthropic-api-key> after"},{"role":"assistant","text":"hello"}]"#;
    const V2_SHA: &str = "7973604f43b515e1af6b1206e85c4fdc88447eed5a7822ca0ec3a77224b792a9";
    const V1_CANONICAL: &str =
        r#"[{"role":"user","text":"hi"},{"role":"assistant","text":"hello"}]"#;
    const V1_SHA: &str = "df991cd9a1ac5c12c32b8cdf0254c3dfbbf26485b505a5afc83a90d1128ebc54";
    let source = br#"{"info":{"id":"pure-partial-store"},"messages":[{"id":"u","type":"user","text":"hi"},{"id":"a-u","type":"assistant","content":[{"id":"r-u","type":"reasoning","text":"before sk-ant-EXAMPLE0000000000000000 after"},{"id":"t-u","type":"text","text":"hello"}]},{"id":"i-u","type":"idle","outcome":"succeeded"}]}"#;
    assert_eq!(hex::encode(Sha256::digest(V2_CANONICAL.as_bytes())), V2_SHA);
    assert_eq!(hex::encode(Sha256::digest(V1_CANONICAL.as_bytes())), V1_SHA);
    let (redacted, preview) = CaptureSnapshotService::preview_redacted_coverage_v2(
        "opencode",
        source.to_vec(),
        CaptureSnapshotPolicy::default(),
    )
    .expect("bounded redacted typed preview");
    assert!(!String::from_utf8_lossy(redacted.bytes()).contains(RAW_SECRET));
    assert_eq!(preview["partial"], false);
    let rows = preview["turns"].as_array().expect("typed turn array");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row["coverage_schema_version"], 2);
    assert_eq!(row["digest"], V2_SHA);
    assert_eq!(row["projection"]["logical_turn_key"], "u");
    assert_eq!(
        row["projection"]["records"],
        serde_json::json!([
            {"type":"user","text":"hi"},
            {"type":"reasoning","provider":"opencode","source_kind":"reasoning",
             "text":"before <REDACTED:anthropic-api-key> after"},
            {"type":"assistant","text":"hello"},
        ])
    );
    let mut legacy = normalize_opencode_export(redacted.bytes());
    redact_turns(&mut legacy);
    assert_eq!(legacy.len(), 1);
    assert_eq!(
        canonical_turn_bytes(&legacy[0].records),
        V1_CANONICAL.as_bytes()
    );
    assert_eq!(legacy[0].digest_hex(), V1_SHA);
    assert_ne!(V1_SHA, V2_SHA);
}

/// The legacy direct normalizer remains frozen while the actual authorized
/// capture/import call graph consumes the private typed runtime companion.
/// COV2-11 replaces COV2-06's temporary preactivation selector here.
#[test]
fn cov2_production_typed_path_uses_v2_api() {
    use libra::internal::ai::observed_agents::{
        Completeness, CoverageSchemaVersion, SemanticRecord, normalize_opencode_export,
        select_coverage_schema,
    };

    let source = br#"{"info":{"id":"ses_inert"},"messages":[{"id":"u","type":"user","text":"hi"},{"id":"a","type":"assistant","content":[{"id":"ap","type":"reasoning","text":"synthetic visible"}]},{"id":"i","type":"idle","outcome":"succeeded"}]}"#;
    let ordinary = std::str::from_utf8(source)
        .expect("synthetic UTF-8")
        .replace(
            r#""type":"reasoning","text":"synthetic visible""#,
            r#""type":"text","text":"ordinary""#,
        );
    let baseline = normalize_opencode_export(ordinary.as_bytes());
    assert_eq!(baseline.len(), 1);
    assert_eq!(
        baseline[0].completeness,
        Completeness::Complete,
        "the control must not be incomplete for an unrelated envelope error"
    );
    let turns = normalize_opencode_export(source);
    assert_eq!(turns.len(), 1);
    assert!(matches!(&turns[0].records[0], SemanticRecord::User { text } if text == "hi"));
    for turn in &turns {
        assert_eq!(turn.completeness, Completeness::Incomplete);
        assert_eq!(select_coverage_schema(turn), CoverageSchemaVersion::V1);
        assert!(
            !turn
                .records
                .iter()
                .any(|record| matches!(record, SemanticRecord::Reasoning { .. }))
        );
    }
    for path in [
        "src/internal/ai/capture/live_checkpoint.rs",
        "src/internal/ai/agent_import.rs",
    ] {
        let code = std::fs::read_to_string(path).expect("read actual production source");
        assert!(
            !code.contains("classify_opencode_coverage_source"),
            "COV2-11 owns production activation: {path}"
        );
    }
    let snapshot =
        std::fs::read_to_string("src/internal/ai/capture/snapshot.rs").expect("snapshot source");
    let legacy = snapshot
        .split("pub fn capture_authorized(")
        .nth(1)
        .expect("legacy capture entry")
        .split("pub(crate) fn capture_authorized_with_reasoning_context")
        .next()
        .expect("legacy body");
    assert!(legacy.contains("LegacyOrdinary"));
    assert!(
        !legacy.contains("preview_redacted_coverage_v2")
            && !legacy.contains("prepare_redacted_coverage_preview")
            && !legacy.contains("classify_opencode_coverage_source")
    );
    let live =
        std::fs::read_to_string("src/internal/ai/capture/live_checkpoint.rs").expect("live source");
    let live = live
        .split("#[cfg(test)]\npub(crate) mod tests")
        .next()
        .expect("production section");
    assert!(
        !live.contains("encode_coverage_turn") && !live.contains("safe_turn_projection_for_schema")
    );
    let import = std::fs::read_to_string("src/internal/ai/agent_import.rs").expect("import source");
    assert_eq!(import.matches("AgentKind::OpenCode => normalize_opencode_export_until(transcript.bytes(), deadline)").count(), 2,
               "the two legacy ordinary preparation projections stay frozen");

    use std::collections::BTreeSet;

    use syn::visit::Visit;
    #[derive(Default)]
    struct Calls(BTreeSet<String>);
    impl<'ast> Visit<'ast> for Calls {
        fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
            if let syn::Expr::Path(path) = call.func.as_ref()
                && let Some(segment) = path.path.segments.last()
            {
                self.0.insert(segment.ident.to_string());
            }
            syn::visit::visit_expr_call(self, call);
        }
        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            self.0.insert(call.method.to_string());
            syn::visit::visit_expr_method_call(self, call);
        }
    }
    for (path, function, required) in [
        (
            "src/internal/ai/capture/live_checkpoint.rs",
            "run_export_stage",
            &["prepare_export_snapshot"][..],
        ),
        (
            "src/internal/ai/capture/live_checkpoint.rs",
            "prepare_export_snapshot",
            &[
                "take_coverage_runtime",
                "reserve_runtime_turn_claims_with_reasoning",
            ][..],
        ),
        (
            "src/internal/ai/agent_import.rs",
            "prepare_import_projection_with_reasoning",
            &["take_coverage_runtime", "encode_redacted_import_runtime"][..],
        ),
        (
            "src/command/agent/import.rs",
            "prepare_candidate_bounded",
            &[
                "decode_redacted_import_runtime",
                "ensure_runtime_import_source_history_until",
            ][..],
        ),
        (
            "src/internal/ai/agent_import.rs",
            "import_prepared_with_subagent_discovery_and_runtime",
            &["partition_reasoning_for_import"][..],
        ),
    ] {
        let code = std::fs::read_to_string(path).expect("actual production source");
        let parsed = syn::parse_file(&code).expect("parse actual source");
        let item = parsed
            .items
            .iter()
            .find_map(|item| match item {
                syn::Item::Fn(item) if item.sig.ident == function => Some(item),
                _ => None,
            })
            .expect("actual top-level production entry exists");
        let mut calls = Calls::default();
        calls.visit_block(&item.block);
        for required in required {
            assert!(
                calls.0.contains(*required),
                "{path}::{function} lost runtime call {required}"
            );
        }
        assert!(
            !calls.0.contains("safe_turn_projection"),
            "typed route must not choose frozen-v1 projection"
        );
    }
}

/// In-memory persistence spies see the actual preview's redacted outputs.
/// Canaries deliberately do not match a secret regex. The ordinary transcript
/// keeps unproven reasoning text and state. Typed coverage does not. No
/// Session, exporter, DB or model.
#[test]
fn cov2_candidate_zero_plaintext_sinks() {
    use libra::internal::ai::capture::snapshot::{CaptureSnapshotPolicy, CaptureSnapshotService};
    const CANARY: &str = "COV10_OPAQUE_CANDIDATE_PAYLOAD_CANARY";
    #[derive(Default)]
    struct MemorySinks {
        ordinary: Vec<Vec<u8>>,
        typed: Vec<serde_json::Value>,
    }
    for include_visible in [false, true] {
        let mut content = vec![
            serde_json::json!({"id":"npart_0123456789abcdef0123456789abcdef","type":"reasoning","text":CANARY,
            "state":{"signature":CANARY,"redactedData":CANARY,"reasoningEncryptedContent":CANARY},
            "originProof":{"nonce":CANARY},"time":{"start":1,"end":2}}),
        ];
        if include_visible {
            content
                .push(serde_json::json!({"id":"good","type":"reasoning","text":"public visible"}));
        }
        content.push(serde_json::json!({"id":"answer","type":"text","text":"done"}));
        let source = serde_json::to_vec(&serde_json::json!({"info":{"id":"ses_sinks"},"messages":[
            {"id":"u","type":"user","text":"prompt"}, {"id":"m","type":"assistant","content":content},
            {"id":"i","type":"idle","outcome":"succeeded"}]})).expect("synthetic source");
        let (ordinary, typed) = CaptureSnapshotService::preview_redacted_coverage_v2(
            "opencode",
            source,
            CaptureSnapshotPolicy::default(),
        )
        .expect("safe partial preview");
        assert_eq!(typed["partial"], true);
        let mut sinks = MemorySinks::default();
        sinks.ordinary.push(ordinary.bytes().to_vec());
        sinks.typed.push(typed);
        assert_eq!(sinks.ordinary.len(), 1);
        assert_eq!(sinks.typed.len(), 1);
        assert!(!sinks.ordinary[0].is_empty());
        let ordinary_text = String::from_utf8(sinks.ordinary[0].clone()).expect("UTF8");
        let decoded: serde_json::Value =
            serde_json::from_slice(&sinks.ordinary[0]).expect("ordinary JSON");
        assert!(
            ordinary_text.contains(CANARY) && decoded.to_string().contains(CANARY),
            "ordinary transcript dropped unproven reasoning fields"
        );
        assert!(
            !sinks.typed[0].to_string().contains(CANARY),
            "typed coverage retained an unproven reasoning field"
        );
        let rows = sinks.typed[0]["turns"]
            .as_array()
            .expect("nonempty turn output");
        assert!(!rows.is_empty());
        assert!(
            rows.iter()
                .all(|row| row["projection"]["completeness"] == "incomplete")
        );
        if include_visible {
            assert_eq!(rows[0]["coverage_schema_version"], 2);
            assert!(rows[0]["projection"].to_string().contains("public visible"));
        } else {
            assert!(rows.iter().all(|row| row["coverage_schema_version"] == 1));
        }
    }
}

#[test]
fn cov2_non_opencode_adapter_unchanged() {
    use libra::internal::ai::{
        capture::snapshot::{CaptureSnapshotPolicy, CaptureSnapshotService},
        observed_agents::{
            Redactor, normalize_claude_transcript, normalize_codex_rollout, redact_turns,
            safe_turn_projection,
        },
    };
    let claude = concat!(
        r#"{"type":"user","message":{"role":"user","content":"hi"}}"#,
        "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"hello"}]}}"#
    );
    let codex = concat!(
        r#"{"type":"event_msg","payload":{"type":"task_started","turn_id":"t"}}"#,
        "\n",
        r#"{"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"hi"}]}}"#,
        "\n",
        r#"{"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"hello"}]}}"#,
        "\n",
        r#"{"type":"event_msg","payload":{"type":"task_complete"}}"#
    );
    for (provider, raw) in [("claude_code", claude), ("codex", codex)] {
        let (baseline, _) = Redactor::new_default().redact(raw.as_bytes());
        let mut old = if provider == "codex" {
            normalize_codex_rollout(baseline.bytes())
        } else {
            normalize_claude_transcript(baseline.bytes())
        };
        redact_turns(&mut old);
        assert!(!old.is_empty(), "control must exercise real normalization");
        let (ordinary, preview) = CaptureSnapshotService::preview_redacted_coverage_v2(
            provider,
            raw.as_bytes().to_vec(),
            CaptureSnapshotPolicy::default(),
        )
        .expect("ordinary preview");
        assert_eq!(ordinary.bytes(), baseline.bytes());
        let rows = preview["turns"].as_array().expect("turns");
        assert_eq!(rows.len(), old.len());
        for (row, turn) in rows.iter().zip(&old) {
            assert_eq!(row["coverage_schema_version"], 1);
            assert_eq!(row["digest"], turn.digest_hex());
            assert_eq!(row["projection"], safe_turn_projection(provider, turn));
        }
    }
}

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

/// CC-07: ordinary Claude thinking stays on the shared normalizer.
/// The fixture is synthetic. It is not a 2.1.303 capture.
#[test]
fn claude_readable_ordinary_retention_matches_the_cc04_matrix() {
    use libra::internal::ai::{
        capture::snapshot::{CaptureSnapshotPolicy, CaptureSnapshotService},
        observed_agents::{
            COVERAGE_SCHEMA_VERSION, ClaudeOrdinaryAssessment, SemanticRecord,
            assess_claude_ordinary_transcript, claude_jsonl_session_matches, coverage_digest_hex,
            normalize_claude_transcript,
        },
    };

    let path = "tests/fixtures/agent_transcripts/claude_code/reasoning.json";
    let raw = std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {path}: {error}"));
    let cases: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&raw).unwrap_or_else(|error| panic!("parse {path}: {error}"));
    let line = |name: &str| -> String {
        cases
            .get(name)
            .and_then(|value| value.as_str())
            .unwrap_or_else(|| panic!("missing case {name}"))
            .to_owned()
    };

    assert_eq!(COVERAGE_SCHEMA_VERSION, 1);
    assert!(!claude_jsonl_session_matches(
        "other-session.jsonl",
        "cc07-session"
    ));
    assert!(claude_jsonl_session_matches(
        "/tmp/cc07/projects/work/cc07-session.jsonl",
        "cc07-session"
    ));
    assert!(!claude_jsonl_session_matches("cc07-session.jsonl", ""));

    let readable_line = line("readable");
    let readable = assess_claude_ordinary_transcript(readable_line.as_bytes());
    assert!(readable.readable_obtained());
    assert_eq!(readable.readable, vec!["CC04-THINKING-CANARY".to_owned()]);
    assert_eq!(readable.signatures, vec!["CC04-SIG".to_owned()]);
    let (ordinary, preview) = CaptureSnapshotService::preview_redacted_coverage_v2(
        "claude_code",
        readable_line.as_bytes().to_vec(),
        CaptureSnapshotPolicy::default(),
    )
    .expect("readable preview");
    let ordinary_text = String::from_utf8(ordinary.bytes().to_vec()).expect("ordinary utf-8");
    assert!(ordinary_text.contains("CC04-THINKING-CANARY"));
    assert!(ordinary_text.contains("CC04-SIG"));
    assert!(ordinary_text.contains("\"uuid\""));
    let report = preview["redaction_report"].to_string();
    assert!(!report.contains("CC04-THINKING-CANARY"));
    assert!(!report.contains("CC04-SIG"));
    let row = &preview["turns"][0];
    assert_eq!(row["coverage_schema_version"], 2);
    let projection = row["projection"].to_string();
    assert!(projection.contains("CC04-THINKING-CANARY"));
    let turns = normalize_claude_transcript(readable_line.as_bytes());
    assert_eq!(turns.len(), 1);
    let digest = coverage_digest_hex(&turns[0].records);
    assert_eq!(row["digest"], digest);
    assert!(matches!(
        &turns[0].records[0],
        SemanticRecord::Reasoning { text, .. } if text.as_str() == "CC04-THINKING-CANARY"
    ));

    let sig_line = line("sig_only");
    let sig_only = assess_claude_ordinary_transcript(sig_line.as_bytes());
    assert!(sig_only.data_obtained());
    assert!(!sig_only.readable_obtained());
    assert!(sig_only.empty_thinking);
    assert_eq!(sig_only.signatures, vec!["CC04-SIG".to_owned()]);
    let (sig_ordinary, sig_preview) = CaptureSnapshotService::preview_redacted_coverage_v2(
        "claude_code",
        sig_line.into_bytes(),
        CaptureSnapshotPolicy::default(),
    )
    .expect("signature preview");
    let sig_text = String::from_utf8(sig_ordinary.bytes().to_vec()).expect("signature utf-8");
    assert!(sig_text.contains("CC04-SIG"));
    assert!(!sig_preview.to_string().contains("CC04-SIG"));

    let missing = assess_claude_ordinary_transcript(line("missing").as_bytes());
    assert!(!missing.data_obtained());
    assert!(missing.missing_block);
    let (_ordinary, missing_preview) = CaptureSnapshotService::preview_redacted_coverage_v2(
        "claude_code",
        line("missing").into_bytes(),
        CaptureSnapshotPolicy::default(),
    )
    .expect("text preview");
    assert_eq!(missing_preview["turns"][0]["coverage_schema_version"], 1);

    let redacted_line = line("redacted");
    let redacted = assess_claude_ordinary_transcript(redacted_line.as_bytes());
    assert!(redacted.data_obtained());
    assert!(!redacted.readable_obtained());
    assert_eq!(redacted.redacted_data, vec!["CC04-DATA".to_owned()]);
    let redacted_turns = normalize_claude_transcript(redacted_line.as_bytes());
    assert!(redacted_turns.is_empty());
    let (redacted_ordinary, _) = CaptureSnapshotService::preview_redacted_coverage_v2(
        "claude_code",
        redacted_line.into_bytes(),
        CaptureSnapshotPolicy::default(),
    )
    .expect("redacted preview");
    let redacted_text =
        String::from_utf8(redacted_ordinary.bytes().to_vec()).expect("redacted utf-8");
    assert!(redacted_text.contains("CC04-DATA"));

    let empty = assess_claude_ordinary_transcript(line("empty_shell").as_bytes());
    assert!(!empty.data_obtained());
    assert!(empty.empty_thinking);
    assert!(empty.empty_attached);

    let dup = assess_claude_ordinary_transcript(line("dup_key").as_bytes());
    assert!(dup.duplicate_records >= 1);
    assert!(!dup.data_obtained());
    assert!(dup.readable.is_empty());

    let escaped_line = line("escape");
    let escaped = assess_claude_ordinary_transcript(escaped_line.as_bytes());
    assert_eq!(escaped.readable, vec!["CC".to_owned()]);
    assert!(escaped_line.contains("\\u0043"));
    assert_ne!(escaped.readable[0], "\\u0043\\u0043");

    let unknown_line = line("unknown_field");
    let unknown = assess_claude_ordinary_transcript(unknown_line.as_bytes());
    assert!(unknown.readable_obtained());
    assert!(unknown.unknown_field);
    assert!(unknown_line.contains("cc04_unknown"));
    let unknown_turns = normalize_claude_transcript(unknown_line.as_bytes());
    assert_eq!(
        unknown_turns[0].completeness,
        libra::internal::ai::observed_agents::Completeness::Incomplete
    );

    let string_content = assess_claude_ordinary_transcript(line("string_content").as_bytes());
    assert!(!string_content.data_obtained());
    assert!(string_content.missing_block);

    let secret = "AKIAABCDEFGHIJKLMNOP";
    let secret_line = format!(
        "{{\"type\":\"assistant\",\"message\":{{\"role\":\"assistant\",\"content\":[{{\"type\":\"thinking\",\"thinking\":\"visible {secret}\",\"signature\":\"CC04-SIG\"}}]}}}}\n"
    );
    let (secret_ordinary, secret_preview) = CaptureSnapshotService::preview_redacted_coverage_v2(
        "claude_code",
        secret_line.into_bytes(),
        CaptureSnapshotPolicy::default(),
    )
    .expect("credential preview");
    let secret_export = format!(
        "{}{}",
        String::from_utf8(secret_ordinary.bytes().to_vec()).expect("secret ordinary utf-8"),
        secret_preview
    );
    assert!(!secret_export.contains(secret));
    assert!(
        !secret_preview["redaction_report"]["matches"]
            .as_array()
            .expect("redaction matches")
            .is_empty()
    );
    assert!(secret_export.contains("visible "));

    let _typed: ClaudeOrdinaryAssessment = readable;
}
