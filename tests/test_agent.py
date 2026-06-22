from __future__ import annotations

from pathlib import Path
import subprocess

import pytest

from spellbook import agent


def test_run_agent_returns_codex_last_message(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    monkeypatch.setattr(agent.shutil, "which", lambda command: "/usr/bin/codex")

    def fake_run(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
        answer_path = Path(command[command.index("--output-last-message") + 1])
        answer_path.write_text("The answer from Codex.\n", encoding="utf-8")
        assert command[:2] == ["/usr/bin/codex", "exec"]
        assert command[-1] == "Explain this repository"
        assert kwargs["cwd"] == tmp_path
        return subprocess.CompletedProcess(command, 0, stdout="", stderr="")

    monkeypatch.setattr(agent.subprocess, "run", fake_run)

    assert agent.run_agent("Explain this repository", cwd=tmp_path) == "The answer from Codex."


def test_run_agent_reports_codex_failure(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(agent.shutil, "which", lambda command: "/usr/bin/codex")
    monkeypatch.setattr(
        agent.subprocess,
        "run",
        lambda command, **kwargs: subprocess.CompletedProcess(
            command, 1, stdout="", stderr="authentication required"
        ),
    )

    with pytest.raises(agent.AgentError, match="authentication required"):
        agent.run_agent("A task")
