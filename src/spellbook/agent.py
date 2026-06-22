"""Non-interactive Codex runner used by Spellbook commands."""

from __future__ import annotations

from pathlib import Path
import shutil
import subprocess
from tempfile import TemporaryDirectory


class AgentError(RuntimeError):
    """Raised when Codex cannot complete a Spellbook request."""


def run_agent(prompt: str, *, cwd: Path | None = None) -> str:
    """Run a task through the authenticated Codex CLI and return its answer."""
    codex_bin = shutil.which("codex")
    if codex_bin is None:
        raise AgentError("Codex CLI was not found. Install it before casting a quest.")

    working_directory = cwd or Path.cwd()
    with TemporaryDirectory(prefix="spellbook-") as temporary_directory:
        answer_path = Path(temporary_directory) / "answer.md"
        result = subprocess.run(
            [
                codex_bin,
                "exec",
                "--ephemeral",
                "--color",
                "never",
                "--sandbox",
                "workspace-write",
                "--output-last-message",
                str(answer_path),
                prompt,
            ],
            cwd=working_directory,
            check=False,
            capture_output=True,
            text=True,
        )

        answer = answer_path.read_text(encoding="utf-8").strip() if answer_path.exists() else ""
        if result.returncode != 0:
            detail = result.stderr.strip() or result.stdout.strip()
            raise AgentError(detail or "Codex could not complete the quest.")
        if not answer:
            raise AgentError("Codex completed without returning an answer.")
        return answer
