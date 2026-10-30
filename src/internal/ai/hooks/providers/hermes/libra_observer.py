#!/usr/bin/env python3
"""Hermes observer scope check.

The same local-CLI rules as the Rust receiver. This file does not open
state.db, does not read a Hermes home, and its process result is not a
tool or message control value. Hook installation belongs to a later card.
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Optional


def admit(payload: dict) -> str:
    platform = payload.get("platform")
    created = payload.get("created_source")
    if not _local_transport(platform, created):
        return "refuse"
    user = payload.get("user_id")
    if not _local_user(user):
        return "refuse"
    cwd = payload.get("cwd")
    root = payload.get("verified_root")
    if not isinstance(cwd, str) or cwd == "" or not isinstance(root, str) or root == "":
        return "refuse"
    cwd_path = Path(cwd)
    root_path = Path(root)
    if not cwd_path.is_absolute() or not root_path.is_absolute():
        return "refuse"
    if _has_dot(cwd_path) or _has_dot(root_path) or cwd_path != root_path:
        return "refuse"
    caller = payload.get("caller_profile")
    if not isinstance(caller, str) or caller == "":
        return "refuse"
    recorded = payload.get("profile_name")
    if isinstance(recorded, str) and recorded != "":
        if recorded != caller:
            return "refuse"
    elif caller != "default":
        return "refuse"
    return "admit"


def _has_dot(path: Path) -> bool:
    return any(part in {".", ".."} for part in path.parts)


def _local_transport(platform, created) -> bool:
    if not isinstance(platform, str) or platform == "":
        return False
    label = platform.lower()
    created_ok = (
        created is None
        or created == ""
        or (isinstance(created, str) and created.lower() == "cli")
    )
    if label == "cli":
        return created_ok
    if label == "local":
        return isinstance(created, str) and created.lower() == "cli"
    return False


def _local_user(user) -> bool:
    if user is None or user == "":
        return True
    if not isinstance(user, str) or not user.startswith("uid:"):
        return False
    geteuid = getattr(os, "geteuid", None)
    if geteuid is None:
        return False
    rest = user[4:]
    if rest == "" or not rest.isdigit():
        return False
    return int(rest) == geteuid()


ALLOWLIST = (
    "source",
    "created_source",
    "user_id",
    "profile_name",
    "turn_id",
    "message_id",
    "phase",
    "outcome",
    "event_id",
)

OBSERVERS = {
    "post_llm_call",
    "on_session_end",
    "on_session_finalize",
    "on_session_reset",
    "agent_loop_stopped",
}


def find_worktree_root(start: str) -> Optional[Path]:
    path = Path(start).resolve()
    for candidate in [path, *path.parents]:
        if (candidate / ".libra").is_dir():
            return candidate
    return None


def hook_stdin(payload: dict) -> Optional[dict]:
    event = payload.get("event")
    if event not in OBSERVERS:
        return None
    cwd = payload.get("cwd")
    root = find_worktree_root(cwd) if isinstance(cwd, str) else None
    if root is None or Path(cwd).resolve() != root:
        return None
    decision = admit(
        {
            "platform": payload.get("source"),
            "created_source": payload.get("created_source"),
            "user_id": payload.get("user_id"),
            "cwd": str(root),
            "profile_name": payload.get("profile_name"),
            "verified_root": str(root),
            "caller_profile": payload.get("profile_name") or "default",
        }
    )
    if decision != "admit":
        return None
    if event == "on_session_reset":
        return None
    frame = {
        "hook_event_name": event,
        "session_id": payload.get("session_id"),
        "cwd": str(root),
    }
    for key in ALLOWLIST:
        if key in payload and payload[key] is not None:
            frame[key] = payload[key]
    frame["event_id"] = "|".join(
        str(payload.get(key) or "")
        for key in ("session_id", "turn_id", "message_id", "phase", "event")
    )
    return frame


def main() -> None:
    if "--admit" in sys.argv:
        payload = json.load(sys.stdin)
        if not isinstance(payload, dict):
            sys.stdout.write("refuse")
            return
        sys.stdout.write(admit(payload))
        return
    if "--hook" not in sys.argv:
        return
    payload = json.load(sys.stdin)
    if not isinstance(payload, dict):
        return
    frame = hook_stdin(payload)
    if frame is None:
        return
    if "--print-frame" in sys.argv:
        sys.stdout.write(json.dumps(frame, sort_keys=True))
        return
    import os
    import subprocess

    binary = os.environ.get("LIBRA_BIN", "libra")
    try:
        subprocess.run(
            [binary, "agent", "hooks", "hermes", "observe"],
            input=json.dumps(frame).encode(),
            cwd=payload.get("cwd") if isinstance(payload.get("cwd"), str) else None,
            env={"PATH": os.environ.get("PATH", ""), "HOME": os.environ.get("HOME", "")},
            timeout=2,
            check=False,
        )
    except subprocess.TimeoutExpired:
        return


if __name__ == "__main__":
    main()
