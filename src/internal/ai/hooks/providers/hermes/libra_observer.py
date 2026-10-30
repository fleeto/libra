#!/usr/bin/env python3
"""Hermes observer scope check.

The same local-CLI rules as the Rust receiver. This file does not open
state.db, does not read a Hermes home, and its process result is not a
tool or message control value. Hook installation belongs to a later card.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def admit(payload: dict) -> str:
    platform = payload.get("platform")
    if not isinstance(platform, str) or platform.lower() != "cli":
        return "refuse"
    created = payload.get("created_source")
    if isinstance(created, str) and created != "" and created.lower() != "cli":
        return "refuse"
    user = payload.get("user_id")
    if isinstance(user, str) and user != "":
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


def main() -> None:
    if "--admit" not in sys.argv:
        return
    payload = json.load(sys.stdin)
    if not isinstance(payload, dict):
        sys.stdout.write("refuse")
        return
    sys.stdout.write(admit(payload))


if __name__ == "__main__":
    main()
