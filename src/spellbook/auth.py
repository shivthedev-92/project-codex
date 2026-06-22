"""Codex authentication helpers.

Authentication is intentionally delegated to the Codex CLI so Spellbook never
handles or stores ChatGPT access tokens.
"""

from __future__ import annotations

import shutil
import subprocess
from collections.abc import Sequence


class CodexCLIError(RuntimeError):
    """Raised when the Codex CLI is unavailable or a command fails."""


def _codex_command(arguments: Sequence[str], *, capture_output: bool) -> subprocess.CompletedProcess[str]:
    codex_bin = shutil.which("codex")
    if codex_bin is None:
        raise CodexCLIError("Codex CLI was not found. Install it before signing in.")

    return subprocess.run(
        [codex_bin, *arguments],
        check=False,
        capture_output=capture_output,
        text=True,
    )


def login_status() -> str:
    """Return the sanitized status emitted by the Codex CLI."""
    result = _codex_command(["login", "status"], capture_output=True)
    raw_output = result.stdout or result.stderr
    output = "\n".join(
        line for line in raw_output.splitlines() if not line.lstrip().startswith("WARNING:")
    ).strip()
    if result.returncode != 0:
        return output or "Not signed in to Codex."
    return output or "Signed in to Codex."


def login() -> None:
    """Start Codex's browser-based ChatGPT login flow."""
    result = _codex_command(["login"], capture_output=False)
    if result.returncode != 0:
        raise CodexCLIError("Codex login did not complete successfully.")


def logout() -> None:
    """Remove credentials through the Codex CLI."""
    result = _codex_command(["logout"], capture_output=False)
    if result.returncode != 0:
        raise CodexCLIError("Codex logout did not complete successfully.")
