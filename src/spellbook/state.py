# ████████╗██╗  ██╗███████╗     ██████╗ ██████╗ ██████╗ ███████╗██╗  ██╗
# ╚══██╔══╝██║  ██║██╔════╝    ██╔════╝██╔═══██╗██╔══██╗██╔════╝╚██╗██╔╝
#    ██║   ███████║█████╗      ██║     ██║   ██║██║  ██║█████╗   ╚███╔╝ 
#    ██║   ██╔══██║██╔══╝      ██║     ██║   ██║██║  ██║██╔══╝   ██╔██╗ 
#    ██║   ██║  ██║███████╗    ╚██████╗╚██████╔╝██████╔╝███████╗██╔╝ ██╗
#    ╚═╝   ╚═╝  ╚═╝╚══════╝     ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝

"""Minimal local state helpers."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

STATE_DIR = Path.home() / ".spellbook"
SESSION_FILE = STATE_DIR / "session.json"


def ensure_state_dir() -> None:
    STATE_DIR.mkdir(parents=True, exist_ok=True)


def load_session() -> dict[str, Any]:
    ensure_state_dir()
    if not SESSION_FILE.exists():
        return {}
    return json.loads(SESSION_FILE.read_text(encoding="utf-8"))


def save_session(data: dict[str, Any]) -> None:
    ensure_state_dir()
    SESSION_FILE.write_text(json.dumps(data, indent=2), encoding="utf-8")
