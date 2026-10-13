//! plan-20260912 MB-08: TUI `d`/`m`/`e` mutation keys on `mega2 browser`.
//!
//! Pins the confirm/cancel rules, file move typing, one POST + one reload per
//! confirmed mutation, hostile destination refusal before any
//! request, and the unchanged CLI surface (no `rmdir`/`mv` subcommands).

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use libra::{
    command::mega2_browser::{
        ActionResult, BrowserState, Editor, Key, perform_delete, perform_move, render,
    },
    internal::protocol::{
        mega2_auth::Mega2Token,
        mega2_mutate::Mega2MutateClient,
        mega2_tree::{ContentType, Listing, ListingEntry, Mega2TreeSession},
    },
};

struct MockServer {
    addr: SocketAddr,
    requests: Arc<AtomicUsize>,
    bodies: Arc<Mutex<Vec<serde_json::Value>>>,
    stop: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}

/// Reads one bounded HTTP request, including the whole declared JSON body.
///
/// A single `read` is not a message boundary: under concurrent command tests
/// the headers and body can arrive in separate TCP reads. The mock must not
/// silently record only whichever request happened to arrive in one packet.
fn read_json_request(stream: &mut TcpStream) -> Option<serde_json::Value> {
    const MAX_REQUEST_BYTES: usize = 32 * 1024;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;

    let mut bytes = Vec::new();
    let mut chunk = [0u8; 4096];
    let header_end = loop {
        if let Some(end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break end;
        }
        let remaining = MAX_REQUEST_BYTES.checked_sub(bytes.len())?;
        if remaining == 0 {
            return None;
        }
        let read_len = remaining.min(chunk.len());
        let n = stream.read(&mut chunk[..read_len]).ok()?;
        if n == 0 {
            return None;
        }
        bytes.extend_from_slice(&chunk[..n]);
    };

    let header = std::str::from_utf8(&bytes[..header_end]).ok()?;
    let content_length = header
        .split("\r\n")
        .skip(1)
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .map(|(_, value)| value.trim().parse::<usize>())
        .transpose()
        .ok()?
        .unwrap_or(0);
    let body_start = header_end.checked_add(4)?;
    let body_end = body_start.checked_add(content_length)?;
    if body_end > MAX_REQUEST_BYTES {
        return None;
    }
    while bytes.len() < body_end {
        let remaining = MAX_REQUEST_BYTES.checked_sub(bytes.len())?;
        let read_len = remaining.min(chunk.len());
        let n = stream.read(&mut chunk[..read_len]).ok()?;
        if n == 0 {
            return None;
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
    serde_json::from_slice(&bytes[body_start..body_end]).ok()
}

impl MockServer {
    /// `entry` responses return the mutate payload; otherwise a tree listing.
    fn start(entry: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        listener.set_nonblocking(true).expect("nonblocking");
        let addr = listener.local_addr().expect("addr");
        let requests = Arc::new(AtomicUsize::new(0));
        let bodies = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let requests_clone = Arc::clone(&requests);
        let bodies_clone = Arc::clone(&bodies);
        let stop_clone = Arc::clone(&stop);
        let join = thread::spawn(move || {
            while !stop_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        if stream.set_nonblocking(false).is_err()
                            || stream
                                .set_write_timeout(Some(Duration::from_secs(5)))
                                .is_err()
                        {
                            continue;
                        }
                        let body = read_json_request(&mut stream);
                        if let Some(body) = body {
                            bodies_clone.lock().expect("lock").push(body);
                        }
                        requests_clone.fetch_add(1, Ordering::SeqCst);
                        let payload = if entry {
                            serde_json::json!({
                                "req_result": true,
                                "data": {"commit_id": "commit-1", "path": "/gone",
                                         "from_path": "/src/old", "to_path": "/dst/old",
                                         "cl_link": null},
                            })
                        } else {
                            serde_json::json!({
                                "req_result": true,
                                "data": {"tree_items": [
                                    {"name": "sub", "path": "/", "content_type": "directory"},
                                    {"name": "readme.txt", "path": "/", "content_type": "file"},
                                ]},
                                "err_message": "",
                            })
                        }
                        .to_string();
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                            payload.len()
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            addr,
            requests,
            bodies,
            stop,
            join: Some(join),
        }
    }

    fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn requests(&self) -> usize {
        self.requests.load(Ordering::SeqCst)
    }

    fn bodies(&self) -> Vec<serde_json::Value> {
        self.bodies.lock().expect("lock").clone()
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn state_with(tree: &MockServer) -> BrowserState {
    let session = Mega2TreeSession::new(&tree.url()).expect("session");
    BrowserState::new(session, "/", None).expect("state")
}

fn mutate_client(server: &MockServer) -> Mega2MutateClient {
    Mega2MutateClient::new(&server.url(), Some(Mega2Token::new("tok").expect("token")))
        .expect("client")
}

#[test]
fn delete_requires_an_extra_confirmation_and_esc_cancels() {
    let tree = MockServer::start(false);
    let mut state = state_with(&tree);
    state.entries = vec![
        ("sub".to_string(), ContentType::Directory),
        ("readme.txt".to_string(), ContentType::File),
    ];
    state.selection = 0;

    assert_eq!(state.handle_key(Key::Delete), ActionResult::Continue);
    assert!(
        matches!(state.editor, Some(Editor::ConfirmDelete(ref n)) if n == "sub"),
        "delete asks for a confirmation line: {:?}",
        state.editor
    );
    let frame = render(&state, "https://example.com");
    assert!(frame.contains("delete 'sub'?"), "{frame}");

    // `n`/Esc cancel without any request.
    assert_eq!(state.handle_key(Key::Other('n')), ActionResult::Continue);
    assert!(state.editor.is_none());
    assert_eq!(state.status.as_deref(), Some("delete cancelled"));

    state.handle_key(Key::Delete);
    assert_eq!(state.handle_key(Key::Cancel), ActionResult::Continue);
    assert!(state.editor.is_none());
    assert_eq!(tree.requests(), 0, "confirmation never touches the network");
}

#[test]
fn file_move_modal_keeps_the_source_selected_while_choosing_a_destination() {
    let tree = MockServer::start(false);
    let session = Mega2TreeSession::new(&tree.url()).expect("session");
    let mut state = BrowserState::new(session, "/artifact", None).expect("state");
    state.entries = vec![
        ("readme.txt".into(), ContentType::File),
        ("destination".into(), ContentType::Directory),
    ];
    assert_eq!(
        state.handle_key(Key::Move),
        ActionResult::LoadMoveDestination {
            path: "/artifact".into()
        }
    );
    let picker = state.move_picker.as_mut().expect("file move picker");
    picker.apply_listing(
        "/artifact".into(),
        Listing {
            entries: vec![ListingEntry {
                name: "destination".into(),
                content_type: ContentType::Directory,
            }],
        },
    );
    assert_eq!(state.handle_key(Key::Down), ActionResult::Continue);
    assert_eq!(
        state.selection, 0,
        "choosing a target must not replace the source"
    );
    assert_eq!(state.handle_key(Key::Enter), ActionResult::Continue);
    assert!(
        state
            .move_picker
            .as_ref()
            .and_then(|picker| picker.confirmed_parent.as_deref())
            .is_some(),
        "the first Enter only chooses the destination"
    );
    assert!(matches!(
        state.handle_key(Key::Enter),
        ActionResult::MoveEntry { ref from_parent, ref from_name, ref to_parent, ref to_name, kind: ContentType::File }
            if from_parent == "/artifact"
                && from_name == "readme.txt"
                && to_parent == "/artifact/destination"
                && to_name == "readme.txt"
    ));
    assert!(state.move_picker.is_none());
    assert_eq!(tree.requests(), 0);
}

#[test]
fn directory_move_modal_does_not_submit_on_the_first_enter() {
    let tree = MockServer::start(false);
    let session = Mega2TreeSession::new(&tree.url()).expect("session");
    let mut state = BrowserState::new(session, "/artifact", None).expect("state");
    state.entries = vec![("source".into(), ContentType::Directory)];
    assert!(matches!(
        state.handle_key(Key::Move),
        ActionResult::LoadMoveDestination { ref path } if path == "/artifact"
    ));
    state.move_picker.as_mut().expect("picker").apply_listing(
        "/artifact".into(),
        Listing {
            entries: vec![ListingEntry {
                name: "destination".into(),
                content_type: ContentType::Directory,
            }],
        },
    );
    state.handle_key(Key::Down);
    assert_eq!(state.handle_key(Key::Enter), ActionResult::Continue);
    assert!(
        state
            .move_picker
            .as_ref()
            .and_then(|picker| picker.confirmed_parent.as_deref())
            .is_some(),
        "destination confirmation remains active"
    );
    assert_eq!(tree.requests(), 0);
}

#[test]
fn files_are_inert_for_delete_and_rename_but_can_be_moved() {
    let tree = MockServer::start(false);
    let session = Mega2TreeSession::new(&tree.url()).expect("session");
    let mut state = BrowserState::new(session, "/artifact", None).expect("state");
    state.entries = vec![("readme.txt".to_string(), ContentType::File)];
    state.selection = 0;

    for key in [Key::Delete, Key::Rename] {
        state.status = None;
        assert_eq!(state.handle_key(key), ActionResult::Continue);
        assert!(state.editor.is_none(), "files must not open an editor");
        assert!(state.status.is_some(), "status explains the refusal");
    }
    assert!(matches!(
        state.handle_key(Key::Move),
        ActionResult::LoadMoveDestination { ref path } if path == "/artifact"
    ));
    assert!(state.move_picker.is_some(), "files open the move picker");
    assert_eq!(tree.requests(), 0);
}

#[test]
fn rename_is_prefilled_same_parent_move_and_hostile_names_are_refused() {
    let tree = MockServer::start(false);
    let mut state = state_with(&tree);
    state.entries = vec![("sub".to_string(), ContentType::Directory)];
    state.selection = 0;

    assert_eq!(state.handle_key(Key::Rename), ActionResult::Continue);
    assert!(
        matches!(state.editor, Some(Editor::RenameDir { ref name, ref input }) if name == "sub" && input == "sub"),
        "rename prefills the current name: {:?}",
        state.editor
    );

    // Hostile new names stay in the editor with a status and no action.
    for hostile in ["/", "a/b", ".."] {
        state.editor = Some(Editor::RenameDir {
            name: "sub".to_string(),
            input: hostile.to_string(),
        });
        assert_eq!(state.handle_key(Key::Enter), ActionResult::Continue);
        assert!(state.editor.is_some(), "hostile {hostile:?} rejected");
        assert!(state.status.is_some());
    }

    // A valid rename is a same-parent move.
    state.editor = Some(Editor::RenameDir {
        name: "sub".to_string(),
        input: "sub2".to_string(),
    });
    let action = state.handle_key(Key::Enter);
    let ActionResult::MoveEntry {
        from_parent,
        from_name,
        to_parent,
        to_name,
        kind,
    } = action
    else {
        panic!("expected MoveEntry, got {action:?}");
    };
    assert_eq!(from_parent, to_parent, "rename keeps the parent");
    assert_eq!(from_name, "sub");
    assert_eq!(to_name, "sub2");
    assert_eq!(kind, ContentType::Directory);
    assert!(state.editor.is_none());
}

#[test]
fn root_level_move_is_refused_before_any_destination_request() {
    let tree = MockServer::start(false);
    let mut state = state_with(&tree);
    state.entries = vec![("sub".to_string(), ContentType::Directory)];
    state.selection = 0;

    assert_eq!(state.handle_key(Key::Move), ActionResult::Continue);
    assert!(
        state
            .status
            .as_deref()
            .is_some_and(|status| status.contains("root-level")),
        "root-level moves are refused with the Mega2 path-policy reason"
    );
    assert!(state.move_picker.is_none());
    assert_eq!(tree.requests(), 0, "refusal is offline");
}

#[tokio::test]
async fn confirmed_delete_and_rename_post_once_then_reload_once() {
    let tree = MockServer::start(false);
    let entry = MockServer::start(true);
    let client = mutate_client(&entry);
    let mut state = state_with(&tree);
    state.fetch_current().await.expect("listing");
    assert_eq!(tree.requests(), 1);

    perform_delete(&mut state, &client, "/", "sub")
        .await
        .expect("delete succeeds");
    assert_eq!(entry.requests(), 1, "one delete POST");
    assert_eq!(tree.requests(), 2, "one reload GET");
    assert_eq!(
        entry.bodies()[0],
        serde_json::json!({"path": "/", "name": "sub", "skip_build": true})
    );

    let mut state2 = state_with(&tree);
    state2.fetch_current().await.expect("listing");
    perform_move(
        &mut state2,
        &client,
        "/",
        "sub",
        "/",
        "sub2",
        ContentType::Directory,
    )
    .await
    .expect("rename succeeds");
    assert_eq!(entry.requests(), 2, "one rename POST");
    assert_eq!(
        entry.bodies()[1],
        serde_json::json!({
            "from_path": "/",
            "from_name": "sub",
            "to_path": "/",
            "to_name": "sub2",
            "skip_build": true,
        })
    );
    assert!(state2.status.is_none(), "success reload clears the status");
}

fn libra(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_libra"))
        .args(args)
        .current_dir(dir)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", dir)
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .output()
        .expect("failed to spawn libra binary")
}

#[test]
fn help_still_exposes_only_the_browser_subcommand() {
    let workdir = tempfile::tempdir().expect("tempdir");

    let parent = libra(workdir.path(), &["mega2", "--help"]);
    assert!(parent.status.success());
    let parent_help = String::from_utf8_lossy(&parent.stdout);
    for forbidden in ["rmdir", "mv ", "delete-entry", "move-entry"] {
        assert!(
            !parent_help.contains(forbidden),
            "help must not invent {forbidden:?}: {parent_help}"
        );
    }

    let browser = libra(workdir.path(), &["mega2", "browser", "--help"]);
    assert!(browser.status.success());
    let browser_help = String::from_utf8_lossy(&browser.stdout);
    for forbidden in ["rmdir", " mv "] {
        assert!(!browser_help.contains(forbidden), "{browser_help}");
    }
}
