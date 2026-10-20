//! Span fields for `libra agent mcp` (plan-20261019 MC-22).
//!
//! One process-global subscriber, same reason as `agent_bridge_span_test`:
//! a thread-local default loses events when the callsite interest was cached
//! first. The cases take a lock so they do not observe each other's spans.

use std::sync::{Mutex, OnceLock};

use libra::internal::ai::mcp_server::protocol::{InboundLine, Session, handle_message};
use serde_json::json;

static SINK: Mutex<Vec<u8>> = Mutex::new(Vec::new());
static CASE_LOCK: Mutex<()> = Mutex::new(());

struct Sink;
impl std::io::Write for Sink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        SINK.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Sink {
    type Writer = Sink;
    fn make_writer(&'a self) -> Self::Writer {
        Sink
    }
}

fn install_subscriber() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_ansi(false)
            .with_writer(Sink)
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("install the process-global span sink");
    });
}

fn capture(line: &str) -> String {
    let _case = CASE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    install_subscriber();
    SINK.lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
    let mut session = Session::with_repository_id("repo-span");
    let _ = handle_message(
        &mut session,
        &InboundLine::Message(line.as_bytes().to_vec()),
    );
    String::from_utf8_lossy(&SINK.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
        .into_owned()
}

fn span_count(captured: &str) -> usize {
    captured.matches("agent.mcp.request").count()
}

#[test]
fn one_request_span_per_request() {
    let captured = capture(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#);
    assert_eq!(
        span_count(&captured),
        1,
        "expected one request span: {captured}"
    );
}

#[test]
fn request_span_method_field() {
    let captured = capture(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#);
    assert!(
        captured.contains("method=\"ping\""),
        "method field missing: {captured}"
    );
    assert!(
        captured.contains("repository_id=repo-span"),
        "repository id missing: {captured}"
    );
}

#[test]
fn unknown_method_is_not_recorded() {
    let marker = "METHOD_MARKER_not_a_known_mcp_method";
    let captured = capture(&format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"{marker}"}}"#
    ));
    assert!(
        captured.contains("method=\"unknown\""),
        "unknown method must be redacted: {captured}"
    );
    assert!(!captured.contains(marker), "raw method leaked: {captured}");
}

#[test]
fn request_id_is_not_recorded() {
    let marker = "ID_MARKER_not_for_spans";
    let captured = capture(&format!(
        r#"{{"jsonrpc":"2.0","id":"{marker}","method":"ping"}}"#
    ));
    assert!(captured.contains("id_hash="), "id hash missing: {captured}");
    assert!(!captured.contains(marker), "raw id leaked: {captured}");
}

#[test]
fn client_info_is_not_recorded() {
    let marker = "CLIENT_MARKER_not_for_spans";
    let line = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": marker, "version": "0"}
        }
    })
    .to_string();
    let captured = capture(&line);
    assert!(
        captured.contains("method=\"initialize\""),
        "initialize method missing: {captured}"
    );
    assert!(!captured.contains(marker), "clientInfo leaked: {captured}");
}

#[test]
fn request_span_excludes_params() {
    let marker = "PARAMS_MARKER_not_for_spans";
    let line = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "ping",
        "params": {"note": marker}
    })
    .to_string();
    let captured = capture(&line);
    assert!(
        !captured.contains(marker),
        "params leaked into the span: {captured}"
    );
}

#[test]
#[serial_test::serial(env)]
fn tools_call_span_has_tool_name() {
    unsafe { std::env::set_var("LIBRA_TEST", "1") };
    let line = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": "libra_test_echo", "arguments": {"text": "ok"}}
    })
    .to_string();
    let captured = capture(&line);
    assert!(
        captured.contains("tool=\"libra_test_echo\""),
        "registered tool name missing: {captured}"
    );
}

#[test]
fn unknown_tool_name_is_not_recorded() {
    let marker = "TOOL_MARKER_not_a_registered_tool";
    let line = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": marker, "arguments": {}}
    })
    .to_string();
    let captured = capture(&line);
    assert!(
        captured.contains("tool=\"unknown\""),
        "unknown tool must be redacted: {captured}"
    );
    assert!(
        !captured.contains(marker),
        "raw tool name leaked: {captured}"
    );
}
