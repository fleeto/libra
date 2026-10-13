//! plan-20260912 MB-02: mega2 browser TUI state-machine integration.
//!
//! Navigation semantics, one-fetch-per-action accounting, hostile-name
//! fail-closed behaviour and rendering back-stops — all against a mock server.

#[cfg(unix)]
use std::{
    io::{Read, Write},
    time::Instant,
};
use std::{
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use libra::{
    command::mega2_browser::{ActionResult, BrowserState, Key, render},
    internal::protocol::mega2_tree::Mega2TreeSession,
};
use serde_json::json;

/// Minimal blocking mock of the mega2 tree route with a canned body.
struct MockTreeServer {
    addr: std::net::SocketAddr,
    request_log: Arc<Mutex<Vec<(String, String)>>>,
    stop: Arc<AtomicBool>,
}

impl MockTreeServer {
    fn start(tree_items: serde_json::Value) -> Self {
        Self::serve(
            move |_| json!({"req_result": true, "data": {"tree_items": tree_items}, "err_message": ""}),
        )
    }

    fn serve(respond: impl Fn(&str) -> serde_json::Value + Send + 'static) -> Self {
        Self::serve_requests(move |target, _| respond(target))
    }

    fn serve_requests(respond: impl Fn(&str, &[u8]) -> serde_json::Value + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock");
        listener.set_nonblocking(true).expect("nonblocking");
        let addr = listener.local_addr().expect("addr");
        let request_log = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let request_log_clone = Arc::clone(&request_log);
        let stop_clone = Arc::clone(&stop);
        thread::spawn(move || {
            while !stop_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .set_nonblocking(false)
                            .expect("blocking mock connection");
                        use std::io::{Read, Write};
                        let mut buf = [0u8; 8192];
                        stream
                            .set_read_timeout(Some(Duration::from_secs(3)))
                            .expect("request deadline");
                        let mut bytes = Vec::new();
                        let (header_end, content_length) = loop {
                            let size = stream.read(&mut buf).expect("request headers");
                            assert!(size > 0, "incomplete request headers");
                            bytes.extend_from_slice(&buf[..size]);
                            assert!(bytes.len() <= 65536, "oversized mock request");
                            if let Some(end) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
                                let headers = String::from_utf8_lossy(&bytes[..end]);
                                let length = headers
                                    .lines()
                                    .filter_map(|line| line.split_once(':'))
                                    .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                                    .map_or(0, |(_, value)| value.trim().parse::<usize>().unwrap());
                                assert!(length <= 65536, "oversized mock body");
                                break (end + 4, length);
                            }
                        };
                        while bytes.len() < header_end + content_length {
                            let size = stream.read(&mut buf).expect("request body");
                            assert!(size > 0, "incomplete request body");
                            bytes.extend_from_slice(&buf[..size]);
                        }
                        let request = String::from_utf8_lossy(&bytes[..header_end]);
                        let mut parts = request
                            .lines()
                            .next()
                            .unwrap_or_default()
                            .split_whitespace();
                        let method = parts.next().unwrap_or_default();
                        let target = parts.next().unwrap_or("/");
                        request_log_clone
                            .lock()
                            .expect("request log")
                            .push((method.to_owned(), target.to_owned()));
                        let body = respond(target, &bytes[header_end..header_end + content_length])
                            .to_string();
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            addr,
            request_log,
            stop,
        }
    }

    fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn requests(&self) -> usize {
        self.request_log.lock().expect("request log").len()
    }
}

impl Drop for MockTreeServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

#[tokio::test]
async fn navigation_root_and_no_prefetch() {
    let server = MockTreeServer::start(json!([
        {"name": "beta-dir", "path": "/", "content_type": "directory"},
        {"name": "alpha-file", "path": "/", "content_type": "file"},
    ]));
    let session = Mega2TreeSession::new(&server.url()).expect("session");
    let mut state = BrowserState::new(session, "/", None).expect("state");

    state.fetch_current().await.expect("root listing");
    assert_eq!(server.requests(), 1, "one request for the initial level");
    assert_eq!(state.entries.len(), 2);

    // Directories were sorted before files by MB-01; selection walks within the
    // current level only and issues no request.
    state.selection = 0;
    assert_eq!(state.handle_key(Key::Enter), ActionResult::FetchCurrent);
    assert_eq!(
        server.requests(),
        1,
        "cursor/enter planning issues no fetch"
    );
    state.fetch_current().await.expect("child listing");
    assert_eq!(server.requests(), 2, "exactly one fetch per navigation");
    assert_eq!(state.path, "/beta-dir");

    // A file entry is inert: no fetch, status line only.
    state.selection = 1;
    assert_eq!(state.handle_key(Key::Enter), ActionResult::Continue);
    assert_eq!(server.requests(), 2);

    // Reload is one fetch; the rendered frame carries the validated listing.
    assert_eq!(state.handle_key(Key::Reload), ActionResult::FetchCurrent);
    state.fetch_current().await.expect("reload");
    assert_eq!(server.requests(), 3);
    let frame = render(&state, &server.url());
    assert!(frame.contains("beta-dir"));
    assert!(frame.contains(&state.path));
}

#[tokio::test]
async fn hostile_names_fail_closed_before_render() {
    let server = MockTreeServer::start(json!([
        {"name": "evil\u{1b}[2Jname", "path": "/", "content_type": "directory"},
    ]));
    let session = Mega2TreeSession::new(&server.url()).expect("session");
    let mut state = BrowserState::new(session, "/", None).expect("state");

    let err = state
        .fetch_current()
        .await
        .expect_err("hostile name refused");
    assert_eq!(
        err.stable_code(),
        libra::utils::error::StableErrorCode::NetworkProtocol
    );
    assert!(
        !err.message().contains("evil"),
        "untrusted payload must not leak into errors"
    );
    assert_eq!(state.entries.len(), 0, "no listing applied on failure");
}

#[test]
fn directory_browser_renders_tree_and_content_panes() {
    let session = Mega2TreeSession::new("http://127.0.0.1:9000").expect("session");
    let state = BrowserState::new(session, "/", None).expect("state");
    let frame = render(&state, "http://127.0.0.1:9000");
    assert!(frame.contains("Directory tree"), "left tree pane: {frame}");
    assert!(
        frame.contains("File content"),
        "right content pane: {frame}"
    );
}

#[cfg(unix)]
struct BrowserPtyChild(Box<dyn portable_pty::Child + Send + Sync>);

#[cfg(unix)]
impl Drop for BrowserPtyChild {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

#[cfg(unix)]
fn wait_for_browser_frame(
    child: &mut BrowserPtyChild,
    transcript: &Mutex<Vec<u8>>,
    start: usize,
    selected: &str,
) {
    wait_for_browser_text(child, transcript, start, &[selected, "q quit"]);
}

#[cfg(unix)]
fn wait_for_browser_text(
    child: &mut BrowserPtyChild,
    transcript: &Mutex<Vec<u8>>,
    start: usize,
    expected: &[&str],
) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let seen =
            String::from_utf8_lossy(&transcript.lock().expect("browser pty transcript")[start..])
                .into_owned();
        if expected.iter().all(|text| seen.contains(text)) && seen.contains("\x1b[0m") {
            return;
        }
        assert!(
            child
                .0
                .try_wait()
                .expect("poll browser pty child")
                .is_none(),
            "browser exited before showing {expected:?}; transcript: {seen:?}"
        );
        assert!(
            Instant::now() < deadline,
            "browser did not show {expected:?}; transcript: {seen:?}"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(unix)]
fn with_browser_pty(
    server: &MockTreeServer,
    scenario: impl FnOnce(
        &mut BrowserPtyChild,
        &Arc<Mutex<Vec<u8>>>,
        &mut dyn Write,
        &dyn portable_pty::MasterPty,
    ),
) {
    with_browser_pty_options(server, &[], false, scenario);
}

#[cfg(unix)]
fn with_browser_pty_options(
    server: &MockTreeServer,
    options: &[&str],
    no_color: bool,
    scenario: impl FnOnce(
        &mut BrowserPtyChild,
        &Arc<Mutex<Vec<u8>>>,
        &mut dyn Write,
        &dyn portable_pty::MasterPty,
    ),
) {
    use portable_pty::{CommandBuilder, PtySize, native_pty_system};

    let test_dir = tempfile::tempdir().expect("isolated browser pty directory");
    let home = test_dir.path().join("home");
    let config_home = home.join(".config");
    std::fs::create_dir_all(&config_home).expect("isolated browser pty config directory");

    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("open browser pty");
    let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_libra"));
    command.args(options);
    command.args(["mega2", "browser", "--server"]);
    command.arg(server.url());
    command.cwd(test_dir.path());
    command.env_clear();
    command.env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin");
    command.env("HOME", &home);
    command.env("USERPROFILE", &home);
    command.env("XDG_CONFIG_HOME", &config_home);
    command.env(
        "LIBRA_CONFIG_GLOBAL_DB",
        home.join(".libra").join("config.db"),
    );
    command.env(
        "LIBRA_CONFIG_SYSTEM_DB",
        home.join(".libra").join("system-config.db"),
    );
    command.env("LANG", "C");
    command.env("LC_ALL", "C");
    command.env("TERM", "xterm");
    if no_color {
        command.env("NO_COLOR", "1");
    }

    let child = pair
        .slave
        .spawn_command(command)
        .expect("spawn browser under pty");
    let mut child = BrowserPtyChild(child);
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().expect("clone pty reader");
    let transcript = Arc::new(Mutex::new(Vec::<u8>::new()));
    let drain_transcript = Arc::clone(&transcript);
    let drain = thread::spawn(move || {
        let mut buffer = [0_u8; 4096];
        while let Ok(read) = reader.read(&mut buffer) {
            if read == 0 {
                break;
            }
            drain_transcript
                .lock()
                .expect("browser pty transcript")
                .extend_from_slice(&buffer[..read]);
        }
    });
    let mut writer = pair.master.take_writer().expect("browser pty writer");

    scenario(
        &mut child,
        &transcript,
        writer.as_mut(),
        pair.master.as_ref(),
    );

    writer.write_all(b"q").expect("send quit key");
    writer.flush().expect("flush quit key");
    let quit_deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        match child.0.try_wait().expect("poll browser after quit") {
            Some(status) => break status,
            None if Instant::now() >= quit_deadline => {
                panic!("browser did not exit after q");
            }
            None => thread::sleep(Duration::from_millis(20)),
        }
    };
    drop(writer);
    drop(pair.master);
    let _ = drain.join();
    assert!(
        String::from_utf8_lossy(&transcript.lock().expect("browser pty transcript"))
            .contains("\x1b[?1049l\x1b[?25h"),
        "browser must restore the terminal on exit"
    );
    assert!(
        status.success(),
        "browser should exit successfully: {status:?}"
    );
}

#[cfg(unix)]
#[test]
fn browser_pty_single_key_dialogs_preserve_text_input_and_cancel_without_writes() {
    let server = MockTreeServer::serve(|target| {
        if target.starts_with("/api/v1/tags/list?") {
            json!({"req_result": true, "data": {"items": [], "total": 0}, "err_message": ""})
        } else if target.starts_with("/api/v1/tree?") {
            json!({"req_result": true, "data": {"tree_items": [
                {"name": "alpha", "path": "/", "content_type": "directory"}
            ]}, "err_message": ""})
        } else {
            json!({"req_result": false, "err_message": "unexpected request"})
        }
    });
    with_browser_pty(&server, |child, transcript, writer, _| {
        wait_for_browser_text(child, transcript, 0, &["a new dir", "e rename", "q quit"]);
        for (input, expected) in [
            (
                &b"a\r"[..],
                &[
                    "┌ New directory",
                    "Error: mega2 entry name must not be empty",
                    "Esc cancel",
                ][..],
            ),
            (
                &b"area-rqjktmde+R"[..],
                &["┌ New directory", "> area-rqjktmde+R▏", "Esc cancel"][..],
            ),
            (&b"\x1b"[..], &["create cancelled", "q quit"][..]),
            (
                &b"e\x7f\x7f\x7f\x7f\x7farea-ae"[..],
                &["┌ Rename directory", "New name", "> area-ae▏", "Esc cancel"][..],
            ),
            (&b"\x1b"[..], &["rename cancelled", "q quit"][..]),
            (
                &b"d"[..],
                &["┌ Delete directory", "delete 'alpha'?", "Enter/y confirm"][..],
            ),
            (&b"\x1b"[..], &["delete cancelled", "q quit"][..]),
            (&b"t"[..], &["mega2 tags", "a create"][..]),
            (
                &b"a\r"[..],
                &["┌ New tag", "Error: mega2 tag name must not be empty"][..],
            ),
            (
                &b"release-ae"[..],
                &["┌ New tag", "Tag name", "> release-ae▏"][..],
            ),
            (
                &b"\r"[..],
                &[
                    "Tag: release-ae",
                    "Message (empty = lightweight)",
                    "Enter create",
                ][..],
            ),
            (
                &b"a release q+R rjkdtm"[..],
                &["> a release q+R rjkdtm▏"][..],
            ),
            (&b"\x1b"[..], &["tag create cancelled"][..]),
            (&b"q"[..], &["Directory tree", "q quit"][..]),
        ] {
            let start = transcript.lock().expect("transcript").len();
            writer
                .write_all(input)
                .expect("send single-key action or text");
            writer.flush().expect("flush input");
            wait_for_browser_text(child, transcript, start, expected);
            if expected.contains(&"Message (empty = lightweight)") {
                let transcript = transcript.lock().expect("transcript");
                let output = String::from_utf8_lossy(&transcript[start..]);
                let frame = output.rsplit("\x1b[2J").next().expect("dialog frame");
                assert!(
                    !frame.contains("Error:"),
                    "corrected name must clear the old error: {frame}"
                );
            }
        }
    });
    assert_eq!(
        server.requests(),
        2,
        "cancelled editors must not write or reload"
    );
    let requests = server.request_log.lock().expect("request log");
    assert!(
        requests.iter().all(|(method, _)| method == "GET"),
        "cancelled editors must not issue write requests: {requests:?}"
    );
    assert!(requests[0].1.starts_with("/api/v1/tree?"));
    assert!(requests[1].1.starts_with("/api/v1/tags/list?"));
}

#[cfg(unix)]
#[test]
fn browser_pty_create_posts_inside_selected_directory_and_reloads_that_branch() {
    for expanded in [false, true] {
        let creates = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
        let captured_creates = Arc::clone(&creates);
        let server = MockTreeServer::serve_requests(move |target, body| {
            let url = url::Url::parse(&format!("http://localhost{target}")).unwrap();
            let data = if url.path() == "/api/v1/create-entry" {
                captured_creates
                    .lock()
                    .unwrap()
                    .push(serde_json::from_slice(body).unwrap());
                json!({"commit_id":"commit-1", "new_oid":"tree-1", "path":"/artifact/rust"})
            } else {
                let path = url.query_pairs().find(|(key, _)| key == "path").unwrap().1;
                match path.as_ref() {
                    "/" => json!({"tree_items":[
                        {"name":"artifact", "content_type":"directory"},
                        {"name":"doc", "content_type":"directory"}
                    ]}),
                    "/artifact" if captured_creates.lock().unwrap().is_empty() => {
                        json!({"tree_items":[{"name":".gitkeep", "content_type":"file"}]})
                    }
                    "/artifact" => json!({"tree_items":[
                        {"name":"rust", "content_type":"directory"},
                        {"name":".gitkeep", "content_type":"file"}
                    ]}),
                    _ => panic!("unexpected request {target}"),
                }
            };
            json!({"req_result":true, "data":data})
        });
        with_browser_pty(&server, |child, transcript, writer, _| {
            wait_for_browser_frame(child, transcript, 0, "> ├─ [+] artifact/");
            if expanded {
                let start = transcript.lock().unwrap().len();
                writer.write_all(b"\x1b[C").unwrap();
                writer.flush().unwrap();
                wait_for_browser_frame(child, transcript, start, "> ├─ [-] artifact/");
            }
            let start = transcript.lock().unwrap().len();
            writer.write_all(b"a").unwrap();
            writer.flush().unwrap();
            wait_for_browser_text(
                child,
                transcript,
                start,
                &["New directory", "Parent: /artifact"],
            );
            let start = transcript.lock().unwrap().len();
            writer.write_all(b"rust\r").unwrap();
            writer.flush().unwrap();
            wait_for_browser_frame(child, transcript, start, "[+] rust/");
            let captured = transcript.lock().unwrap().clone();
            let text = String::from_utf8_lossy(&captured[start..]);
            let frame = text.split("\x1b[2J\x1b[H").last().unwrap();
            assert!(
                frame.contains("> ├─ [-] artifact/"),
                "selection retained: {frame}"
            );
            assert!(frame.contains("[+] doc/"), "sibling retained: {frame}");
        });
        assert_eq!(
            *creates.lock().unwrap(),
            vec![json!({
                "path":"/artifact", "name":"rust", "is_directory":true,
                "content":null, "skip_build":true
            })]
        );
        let requests = server.request_log.lock().unwrap();
        assert_eq!(requests.len(), if expanded { 4 } else { 3 });
        let post = &requests[requests.len() - 2];
        assert_eq!(post, &("POST".into(), "/api/v1/create-entry".into()));
        let (method, target) = requests.last().unwrap();
        assert_eq!(method, "GET");
        let url = url::Url::parse(&format!("http://localhost{target}")).unwrap();
        assert_eq!(url.path(), "/api/v1/tree");
        assert!(
            url.query_pairs()
                .any(|(key, value)| key == "path" && value == "/artifact")
        );
    }
}

#[cfg(unix)]
#[test]
fn browser_pty_file_move_uses_a_separate_destination_picker_and_second_confirmation() {
    let moves = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
    let captured_moves = Arc::clone(&moves);
    let server = MockTreeServer::serve_requests(move |target, body| {
        let url = url::Url::parse(&format!("http://localhost{target}")).expect("mock URL");
        let data = match url.path() {
            "/api/v1/move-entry" => {
                captured_moves
                    .lock()
                    .expect("move captures")
                    .push(serde_json::from_slice(body).expect("move body"));
                json!({
                    "commit_id": "commit-1",
                    "from_path": "/artifact/readme.txt",
                    "to_path": "/artifact/destination/readme.txt",
                    "cl_link": null,
                })
            }
            "/api/v1/tree" => {
                let path = url
                    .query_pairs()
                    .find(|(key, _)| key == "path")
                    .expect("tree path")
                    .1;
                match path.as_ref() {
                    "/" => json!({"tree_items": [
                        {"name": "artifact", "content_type": "directory"}
                    ]}),
                    "/artifact" if captured_moves.lock().expect("move captures").is_empty() => {
                        json!({"tree_items": [
                            {"name": "destination", "content_type": "directory"},
                            {"name": "readme.txt", "content_type": "file"}
                        ]})
                    }
                    "/artifact" => json!({"tree_items": [
                        {"name": "destination", "content_type": "directory"}
                    ]}),
                    "/artifact/destination" => json!({"tree_items": []}),
                    other => panic!("unexpected tree path {other}"),
                }
            }
            "/api/v1/blob" => json!("file preview\n"),
            other => panic!("unexpected request {other}"),
        };
        json!({"req_result": true, "data": data, "err_message": ""})
    });

    with_browser_pty(&server, |child, transcript, writer, _| {
        wait_for_browser_frame(child, transcript, 0, "> └─ [+] artifact/");
        let start = transcript.lock().expect("transcript").len();
        writer
            .write_all(b"\x1b[C\x1b[B\x1b[B")
            .expect("expand and select file");
        writer.flush().expect("flush expand and select file");
        wait_for_browser_text(child, transcript, start, &["readme.txt", "file preview"]);

        let start = transcript.lock().expect("transcript").len();
        writer.write_all(b"m").expect("open move picker");
        writer.flush().expect("flush move picker");
        wait_for_browser_text(
            child,
            transcript,
            start,
            &[
                "Move file",
                "Source: /artifact/readme.txt",
                "Destination within /artifact: /artifact",
                "Enter choose | Right open",
            ],
        );
        assert!(
            moves.lock().expect("moves").is_empty(),
            "opening is read-only"
        );

        let start = transcript.lock().expect("transcript").len();
        writer
            .write_all(b"\x1b[B\x1b[C")
            .expect("browse destination");
        writer.flush().expect("flush destination browse");
        wait_for_browser_text(
            child,
            transcript,
            start,
            &["Destination within /artifact: /artifact/destination"],
        );
        assert!(
            moves.lock().expect("moves").is_empty(),
            "Right only opens a folder"
        );

        let start = transcript.lock().expect("transcript").len();
        writer.write_all(b"\r").expect("choose destination");
        writer.flush().expect("flush destination choice");
        wait_for_browser_text(
            child,
            transcript,
            start,
            &[
                "Destination: /artifact/destination",
                "Enter move | Esc back",
            ],
        );
        assert!(
            moves.lock().expect("moves").is_empty(),
            "first Enter does not write"
        );

        let start = transcript.lock().expect("transcript").len();
        writer.write_all(b"\r").expect("confirm move");
        writer.flush().expect("flush move confirmation");
        wait_for_browser_text(child, transcript, start, &["q quit"]);
        let deadline = Instant::now() + Duration::from_secs(5);
        while moves.lock().expect("moves").is_empty() {
            assert!(Instant::now() < deadline, "move request was not sent");
            thread::sleep(Duration::from_millis(20));
        }
    });
    assert_eq!(
        *moves.lock().expect("moves"),
        vec![json!({
            "from_path": "/artifact",
            "from_name": "readme.txt",
            "to_path": "/artifact/destination",
            "to_name": "readme.txt",
            "is_directory": false,
            "skip_build": true,
        })]
    );
}

#[cfg(unix)]
#[test]
fn browser_pty_directory_move_uses_the_picker_and_preserves_directory_type() {
    let moves = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
    let captured_moves = Arc::clone(&moves);
    let server = MockTreeServer::serve_requests(move |target, body| {
        let url = url::Url::parse(&format!("http://localhost{target}")).expect("mock URL");
        let data = match url.path() {
            "/api/v1/move-entry" => {
                captured_moves
                    .lock()
                    .expect("move captures")
                    .push(serde_json::from_slice(body).expect("move body"));
                json!({
                    "commit_id": "commit-1",
                    "from_path": "/artifact/rust",
                    "to_path": "/artifact/destination/rust",
                    "cl_link": null,
                })
            }
            "/api/v1/tree" => {
                let path = url
                    .query_pairs()
                    .find(|(key, _)| key == "path")
                    .expect("tree path")
                    .1;
                match path.as_ref() {
                    "/" => json!({"tree_items": [
                        {"name": "artifact", "content_type": "directory"}
                    ]}),
                    "/artifact" if captured_moves.lock().expect("move captures").is_empty() => {
                        json!({"tree_items": [
                            {"name": "destination", "content_type": "directory"},
                            {"name": "rust", "content_type": "directory"}
                        ]})
                    }
                    "/artifact" => json!({"tree_items": [
                        {"name": "destination", "content_type": "directory"}
                    ]}),
                    "/artifact/destination" => json!({"tree_items": []}),
                    other => panic!("unexpected tree path {other}"),
                }
            }
            other => panic!("unexpected request {other}"),
        };
        json!({"req_result": true, "data": data, "err_message": ""})
    });

    with_browser_pty(&server, |child, transcript, writer, _| {
        wait_for_browser_frame(child, transcript, 0, "> └─ [+] artifact/");
        let start = transcript.lock().expect("transcript").len();
        writer
            .write_all(b"\x1b[C\x1b[B\x1b[B")
            .expect("expand and select directory");
        writer.flush().expect("flush expand and select directory");
        wait_for_browser_frame(child, transcript, start, ">   └─ [+] rust/");

        let start = transcript.lock().expect("transcript").len();
        writer.write_all(b"m").expect("open move picker");
        writer.flush().expect("flush move picker");
        wait_for_browser_text(
            child,
            transcript,
            start,
            &[
                "Move directory",
                "Source: /artifact/rust",
                "Destination within /artifact: /artifact",
            ],
        );
        assert!(
            moves.lock().expect("moves").is_empty(),
            "opening is read-only"
        );

        let start = transcript.lock().expect("transcript").len();
        writer
            .write_all(b"\x1b[B\x1b[C")
            .expect("browse destination");
        writer.flush().expect("flush destination browse");
        wait_for_browser_text(
            child,
            transcript,
            start,
            &["Destination within /artifact: /artifact/destination"],
        );
        assert!(
            moves.lock().expect("moves").is_empty(),
            "Right only opens a folder"
        );

        let start = transcript.lock().expect("transcript").len();
        writer.write_all(b"\r").expect("choose destination");
        writer.flush().expect("flush destination choice");
        wait_for_browser_text(
            child,
            transcript,
            start,
            &[
                "Destination: /artifact/destination",
                "Enter move | Esc back",
            ],
        );
        assert!(
            moves.lock().expect("moves").is_empty(),
            "first Enter does not write"
        );

        let start = transcript.lock().expect("transcript").len();
        writer.write_all(b"\r").expect("confirm move");
        writer.flush().expect("flush move confirmation");
        wait_for_browser_text(child, transcript, start, &["q quit"]);
        let deadline = Instant::now() + Duration::from_secs(5);
        while moves.lock().expect("moves").is_empty() {
            assert!(Instant::now() < deadline, "move request was not sent");
            thread::sleep(Duration::from_millis(20));
        }
    });
    assert_eq!(
        *moves.lock().expect("moves"),
        vec![json!({
            "from_path": "/artifact",
            "from_name": "rust",
            "to_path": "/artifact/destination",
            "to_name": "rust",
            "skip_build": true,
        })],
        "directory moves preserve Mega2's default is_directory=true by omitting the field"
    );
}

#[cfg(unix)]
#[test]
fn browser_pty_down_arrow_moves_selection_without_quitting() {
    let server = MockTreeServer::start(json!([
        {"name": "alpha", "path": "/", "content_type": "directory"},
        {"name": "beta", "path": "/", "content_type": "directory"},
    ]));
    with_browser_pty(&server, |child, transcript, writer, _| {
        wait_for_browser_frame(child, transcript, 0, "> ├─ [+] alpha/");
        for (input, selected) in [
            (&b"\x1b[B"[..], "> └─ [+] beta/"),
            (&b"\x1b[B"[..], "> └─ [+] beta/"),
            (&b"\x1b[A"[..], "> ├─ [+] alpha/"),
        ] {
            let start = transcript.lock().unwrap().len();
            writer.write_all(input).unwrap();
            writer.flush().unwrap();
            wait_for_browser_frame(child, transcript, start, selected);
        }
        assert!(child.0.try_wait().unwrap().is_none());
        assert_eq!(
            server.requests(),
            1,
            "directory cursor movement must not fetch"
        );
    });
}

#[cfg(unix)]
#[test]
fn browser_pty_tree_preview_scroll_resize_and_collapse() {
    let server = MockTreeServer::serve(|target| {
        let url = url::Url::parse(&format!("http://localhost{target}")).unwrap();
        let path = url
            .query_pairs()
            .find(|(key, _)| key == "path")
            .unwrap()
            .1
            .into_owned();
        let data = match (url.path(), path.as_str()) {
            ("/api/v1/tree", "/") => json!({"tree_items": [
                {"name":"src", "content_type":"directory"}, {"name":"README.md", "content_type":"file"}
            ]}),
            ("/api/v1/tree", "/src") => {
                json!({"tree_items": [{"name":"main.rs", "content_type":"file"}]})
            }
            ("/api/v1/blob", "/src/main.rs") => json!(
                (1..=80)
                    .map(|n| format!("PREVIEW_LINE_{n:02}\n"))
                    .collect::<String>()
            ),
            ("/api/v1/blob", "/README.md") => json!("ROOT_README\n\u{1b}[2Jsafe"),
            _ => panic!("unexpected request {target}"),
        };
        json!({"req_result":true, "data":data})
    });
    with_browser_pty(&server, |child, transcript, writer, master| {
        wait_for_browser_frame(child, transcript, 0, "> ├─ [+] src/");
        for (input, expected) in [
            (&b"\x1b[C"[..], "> ├─ [-] src/"),
            (&b"\x1b[B"[..], "PREVIEW_LINE_01"),
            (&b"\t\x1b[6~"[..], "PREVIEW_LINE_17"),
        ] {
            let start = transcript.lock().unwrap().len();
            writer.write_all(input).unwrap();
            writer.flush().unwrap();
            wait_for_browser_frame(child, transcript, start, expected);
        }
        assert_eq!(
            server.requests(),
            3,
            "one root, one branch, one selected file; scrolling is local"
        );
        let start = transcript.lock().unwrap().len();
        master
            .resize(portable_pty::PtySize {
                rows: 12,
                cols: 60,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        wait_for_browser_frame(child, transcript, start, "PREVIEW_LINE_17");
        let captured = transcript.lock().unwrap().clone();
        let text = String::from_utf8_lossy(&captured[start..]);
        let frame = text
            .split("\x1b[2J\x1b[H")
            .last()
            .unwrap()
            .split("\x1b[0m")
            .next()
            .unwrap();
        assert!(
            frame.lines().count() <= 11,
            "resized frame overflows: {frame:?}"
        );
        for (input, expected) in [
            (&b"\t\x1b[D"[..], "> ├─ [-] src/"),
            (&b"\x1b[D"[..], "> ├─ [+] src/"),
            (&b"\x1b[B"[..], "ROOT_README"),
        ] {
            let start = transcript.lock().unwrap().len();
            writer.write_all(input).unwrap();
            writer.flush().unwrap();
            wait_for_browser_frame(child, transcript, start, expected);
        }
        assert_eq!(server.requests(), 4);
        let captured = transcript.lock().unwrap().clone();
        let text = String::from_utf8_lossy(&captured);
        let last = text.split("\x1b[2J\x1b[H").last().unwrap();
        assert!(
            last.contains("?[2Jsafe"),
            "file control codes are neutralized"
        );
        assert!(
            !last.contains("PREVIEW_LINE"),
            "previous file must not leak into new preview"
        );
    });
}

#[cfg(unix)]
#[test]
fn browser_pty_preview_failure_stays_open_and_next_file_recovers() {
    let server = MockTreeServer::serve(|target| {
        let url = url::Url::parse(&format!("http://localhost{target}")).unwrap();
        let data = if url.path() == "/api/v1/tree" {
            json!({"tree_items": [{"name":"bad.txt", "content_type":"file"}, {"name":"good.txt", "content_type":"file"}]})
        } else if url
            .query_pairs()
            .any(|(key, value)| key == "path" && value == "/good.txt")
        {
            json!("RECOVERED_FILE_CONTENT")
        } else {
            serde_json::Value::Null
        };
        json!({"req_result":true, "data":data, "err_message":"SECRET_SERVER_BODY"})
    });
    with_browser_pty(&server, |child, transcript, writer, _| {
        wait_for_browser_frame(child, transcript, 0, "Preview unavailable");
        assert!(child.0.try_wait().unwrap().is_none());
        let start = transcript.lock().unwrap().len();
        writer.write_all(b"\x1b[B").unwrap();
        writer.flush().unwrap();
        wait_for_browser_frame(child, transcript, start, "RECOVERED_FILE_CONTENT");
        let captured = transcript.lock().unwrap().clone();
        let text = String::from_utf8_lossy(&captured);
        assert!(!text.contains("SECRET_SERVER_BODY"));
        assert!(
            !text
                .split("\x1b[2J\x1b[H")
                .last()
                .unwrap()
                .contains("Preview unavailable")
        );
        assert_eq!(server.requests(), 3);
    });
}

#[cfg(unix)]
#[test]
fn browser_pty_file_syntax_colors_respect_global_color_options() {
    let server = MockTreeServer::serve(|target| {
        let data = if target.starts_with("/api/v1/tree?") {
            json!({"tree_items": [{"name":"main.rs", "content_type":"file"}, {"name":"plain.txt", "content_type":"file"}]})
        } else if target.contains("main.rs") {
            json!("fn main() { let text = \"中文\"; }\n// SYNTAX_LOADED")
        } else {
            json!("PLAIN_TEXT fn not_code")
        };
        json!({"req_result":true, "data":data})
    });
    for (options, no_color, expected_color) in [
        (&[][..], false, true),
        (&["--no-color"][..], false, false),
        (&["--color=never"][..], false, false),
        (&[][..], true, false),
        (&["--color=always"][..], true, true),
    ] {
        with_browser_pty_options(
            &server,
            options,
            no_color,
            |child, transcript, writer, _| {
                wait_for_browser_frame(child, transcript, 0, "SYNTAX_LOADED");
                let captured = transcript.lock().unwrap().clone();
                let text = String::from_utf8_lossy(&captured);
                let frame = text.split("\x1b[2J\x1b[H").last().unwrap();
                assert_eq!(frame.contains("\x1b[35mfn"), expected_color);
                assert_eq!(frame.contains("\x1b[32m"), expected_color);
                if !expected_color {
                    assert!(!frame.contains("\x1b[3"));
                }
                let start = transcript.lock().unwrap().len();
                writer.write_all(b"\x1b[B").unwrap();
                writer.flush().unwrap();
                wait_for_browser_frame(child, transcript, start, "PLAIN_TEXT fn not_code");
                let captured = transcript.lock().unwrap().clone();
                let text = String::from_utf8_lossy(&captured[start..]);
                let frame = text.split("\x1b[2J\x1b[H").last().unwrap();
                assert!(
                    !frame.contains("\x1b[3"),
                    "plain text must not retain previous syntax colors"
                );
            },
        );
    }
    assert_eq!(
        server.requests(),
        15,
        "coloring never adds network requests"
    );
}
