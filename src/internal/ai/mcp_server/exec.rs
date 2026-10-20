//! Read-tool subprocess executor (plan-20261019 MC-29).
//!
//! Each read tool runs in `libra agent mcp --read-worker`. The parent replies
//! with `LBR-AGENT-036` at the read deadline, kills the worker's process
//! group, and only then releases the concurrency slot.

use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::{Condvar, Mutex},
    time::{Duration, Instant},
};

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, BufReader};

use super::{output, protocol, tools};
use crate::utils::error::{CliError, CliResult};

const READ_SLOTS: usize = 4;
const REAP_BUDGET: Duration = Duration::from_secs(1);

struct SlotGate {
    free: Mutex<usize>,
    cv: Condvar,
}

impl SlotGate {
    fn acquire(&self) -> SlotGuard<'_> {
        let mut free = self
            .free
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while *free == 0 {
            free = self
                .cv
                .wait(free)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        *free -= 1;
        SlotGuard { gate: self }
    }

    fn release(&self) {
        let mut free = self
            .free
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *free += 1;
        self.cv.notify_one();
    }
}

struct SlotGuard<'a> {
    gate: &'a SlotGate,
}

impl Drop for SlotGuard<'_> {
    fn drop(&mut self) {
        self.gate.release();
    }
}

fn running_as_libra_binary() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().starts_with("libra"))
        })
        .unwrap_or(false)
}

fn invoke_locally(name: &str, arguments: &Value) -> Result<Value, String> {
    match tools::invoke_in_worker(name, arguments) {
        Ok(structured) => Ok(protocol::tool_success_result(&structured)),
        Err(message) => Ok(protocol::tool_error_result(
            "LBR-AGENT-027",
            &message,
            false,
        )),
    }
}

fn slots() -> &'static SlotGate {
    static SLOTS: std::sync::OnceLock<SlotGate> = std::sync::OnceLock::new();
    SLOTS.get_or_init(|| SlotGate {
        free: Mutex::new(READ_SLOTS),
        cv: Condvar::new(),
    })
}

pub fn is_read_worker_argv(argv: &[std::ffi::OsString]) -> bool {
    let mut tokens = argv.iter().skip(1).filter_map(|arg| arg.to_str());
    tokens.next() == Some("agent")
        && tokens.next() == Some("mcp")
        && tokens.any(|token| token == "--read-worker")
}

pub fn read_deadline() -> Duration {
    #[cfg(debug_assertions)]
    if std::env::var("LIBRA_TEST").ok().as_deref() == Some("1")
        && let Ok(raw) = std::env::var("LIBRA_MCP_TEST_READ_DEADLINE_MS")
        && let Ok(ms) = raw.parse::<u64>()
        && ms > 0
    {
        return Duration::from_millis(ms);
    }
    output::READ_DEADLINE
}

/// Run one read tool in a worker process. The returned value is a tool result.
pub fn execute_read_tool(name: &str, arguments: &Value) -> Result<Value, String> {
    // Library callers inside a test binary are not the `libra` executable.
    // The stdio server is, and that is the process which spawns workers.
    if !running_as_libra_binary() {
        return invoke_locally(name, arguments);
    }
    let _slot = slots().acquire();
    let exe =
        std::env::current_exe().map_err(|_| "read worker binary is not available".to_string())?;
    let mut command = Command::new(exe);
    command
        .arg("agent")
        .arg("mcp")
        .arg("--read-worker")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .spawn()
        .map_err(|_| "read worker could not be started".to_string())?;
    let request = json!({"name": name, "arguments": arguments});
    let bytes = serde_json::to_vec(&request)
        .map_err(|_| "read worker request could not be encoded".to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&bytes)
            .map_err(|_| "read worker request could not be written".to_string())?;
        stdin
            .write_all(b"\n")
            .map_err(|_| "read worker request could not be written".to_string())?;
    }
    wait_for_worker(&mut child)
}

fn wait_for_worker(child: &mut std::process::Child) -> Result<Value, String> {
    let deadline = read_deadline();
    let started = Instant::now();
    // Drain stdout while the worker runs. A full pipe would stall a large
    // result, and the parent would then mistake that stall for the deadline.
    let stdout = child.stdout.take();
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = stdout {
            let _ = pipe.read_to_end(&mut bytes);
        }
        bytes
    });
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let bytes = reader.join().unwrap_or_default();
                return if status.success() {
                    parse_worker_result(&bytes)
                } else {
                    Ok(protocol::tool_error_result(
                        "LBR-AGENT-037",
                        "read worker failed before producing a result",
                        false,
                    ))
                };
            }
            Ok(None) if started.elapsed() >= deadline => {
                kill_worker(child);
                let _ = reader.join();
                return Ok(protocol::tool_error_result(
                    "LBR-AGENT-036",
                    "read tool exceeded the 30 s deadline",
                    true,
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(_) => {
                kill_worker(child);
                let _ = reader.join();
                return Ok(protocol::tool_error_result(
                    "LBR-AGENT-037",
                    "read worker could not be observed",
                    false,
                ));
            }
        }
    }
}

fn kill_worker(child: &mut std::process::Child) {
    #[cfg(unix)]
    // SAFETY: `child.id()` is the worker started with `process_group(0)`, so
    // its process-group id is its pid. A negative pid asks `kill` to signal
    // that group. The child is still unreaped here.
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.kill();
    let started = Instant::now();
    while started.elapsed() < REAP_BUDGET {
        if matches!(child.try_wait(), Ok(Some(_)) | Err(_)) {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let _ = child.wait();
}

fn parse_worker_result(stdout: &[u8]) -> Result<Value, String> {
    let limited = if stdout.len() > 2 * 1024 * 1024 {
        &stdout[..2 * 1024 * 1024]
    } else {
        stdout
    };
    let text = String::from_utf8(limited.to_vec())
        .map_err(|_| "read worker output was not UTF-8".to_string())?;
    let parsed: Value = serde_json::from_str(text.trim())
        .map_err(|_| "read worker output was not JSON".to_string())?;
    if parsed.get("ok").and_then(Value::as_bool) == Some(true) {
        let structured = parsed.get("structured").cloned().unwrap_or(Value::Null);
        return Ok(protocol::tool_success_result(&structured));
    }
    let code = parsed
        .get("stable_code")
        .and_then(Value::as_str)
        .unwrap_or("LBR-AGENT-037");
    let message = parsed
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("read worker failed");
    let retryable = parsed
        .get("retryable")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Ok(protocol::tool_error_result(code, message, retryable))
}

pub fn line_starts_read_tool(line: &protocol::InboundLine) -> bool {
    let protocol::InboundLine::Message(bytes) = line else {
        return false;
    };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let Ok(parsed) = serde_json::from_str::<Value>(text) else {
        return false;
    };
    if parsed.get("method").and_then(Value::as_str) != Some("tools/call") {
        return false;
    }
    let Some(name) = parsed
        .get("params")
        .and_then(|params| params.get("name"))
        .and_then(Value::as_str)
    else {
        return false;
    };
    tools::registered_tools()
        .iter()
        .any(|tool| tool.name == name && tool.read_only)
}

/// Hidden worker: one server-built request in, one JSON result out.
pub async fn run_read_worker() -> CliResult<()> {
    let storage = crate::utils::util::try_get_storage_path(None)
        .map_err(|_| CliError::fatal("read worker could not find a repository".to_string()))?;
    if let Err(message) = prepare_read_only_repository(&storage).await {
        write_worker_error("LBR-AGENT-037", &message, false)?;
        return Ok(());
    }
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut line = String::new();
    let read = stdin.read_line(&mut line).await.map_err(|error| {
        CliError::fatal(format!("read worker could not read its request: {error}"))
    })?;
    if read == 0 {
        write_worker_error(
            "LBR-AGENT-037",
            "read worker received an empty request",
            false,
        )?;
        return Ok(());
    }
    let request: Value = serde_json::from_str(&line)
        .map_err(|_| CliError::fatal("read worker request was not JSON".to_string()))?;
    let name = request.get("name").and_then(Value::as_str).unwrap_or("");
    let arguments = request
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    match tools::invoke_in_worker(name, &arguments) {
        Ok(structured) => write_worker_success(&structured),
        Err(message) => write_worker_error("LBR-AGENT-027", &message, false),
    }
}

async fn prepare_read_only_repository(storage: &Path) -> Result<(), String> {
    let db_path = storage.join(crate::utils::util::DATABASE);
    let conn = crate::internal::db::open_database_without_migrations(&db_path)
        .await
        .map_err(|_| "read worker could not open the repository database".to_string())?;
    let current = crate::internal::db::migration::current_builtin_schema_version_readonly(&conn)
        .await
        .map_err(|_| "read worker could not read the repository schema version".to_string())?;
    let latest = crate::internal::db::migration::latest_builtin_schema_version()
        .map_err(|_| "read worker could not determine its schema version".to_string())?;
    if current.is_none() || current != latest {
        return Err("repository schema is not the version this binary reads".to_string());
    }
    let object_format = read_object_format(&conn).await?;
    let hash_kind = crate::internal::object_format::parse_config_value(&object_format)
        .map_err(|_| format!("unsupported object format: '{object_format}'"))?;
    git_internal::hash::set_hash_kind(hash_kind);
    Ok(())
}

async fn read_object_format(conn: &sea_orm::DatabaseConnection) -> Result<String, String> {
    use sea_orm::{ConnectionTrait, Statement};
    let row = conn
        .query_one_raw(Statement::from_sql_and_values(
            conn.get_database_backend(),
            "SELECT value FROM config_kv WHERE key = ? ORDER BY id DESC LIMIT 1",
            ["core.objectformat".into()],
        ))
        .await
        .map_err(|_| "read worker could not read core.objectformat".to_string())?;
    match row {
        Some(row) => row
            .try_get_by_index::<String>(0)
            .map_err(|_| "read worker could not decode core.objectformat".to_string()),
        None => Ok("sha1".to_string()),
    }
}

fn write_worker_success(structured: &Value) -> CliResult<()> {
    let line =
        serde_json::to_string(&json!({"ok": true, "structured": structured})).map_err(|error| {
            CliError::fatal(format!("read worker could not encode its result: {error}"))
        })?;
    println!("{line}");
    Ok(())
}

fn write_worker_error(code: &str, message: &str, retryable: bool) -> CliResult<()> {
    let line = serde_json::to_string(&json!({
        "ok": false,
        "stable_code": code,
        "message": message,
        "retryable": retryable
    }))
    .map_err(|error| CliError::fatal(format!("read worker could not encode its error: {error}")))?;
    println!("{line}");
    Ok(())
}
