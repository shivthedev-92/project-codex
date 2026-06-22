from __future__ import annotations

import subprocess

import pytest

from spellbook import auth


def test_login_status_uses_codex_cli(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(auth.shutil, "which", lambda command: "/usr/bin/codex")

    def fake_run(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        assert args[0] == ["/usr/bin/codex", "login", "status"]
        assert kwargs["capture_output"] is True
        return subprocess.CompletedProcess(
            args[0],
            0,
            stdout="",
            stderr="WARNING: harmless setup warning\nLogged in using ChatGPT\n",
        )

    monkeypatch.setattr(auth.subprocess, "run", fake_run)

    assert auth.login_status() == "Logged in using ChatGPT"


def test_login_inherits_terminal_for_browser_flow(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(auth.shutil, "which", lambda command: "/usr/bin/codex")

    def fake_run(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        assert args[0] == ["/usr/bin/codex", "login"]
        assert kwargs["capture_output"] is False
        return subprocess.CompletedProcess(args[0], 0)

    monkeypatch.setattr(auth.subprocess, "run", fake_run)

    auth.login()


def test_missing_codex_cli_has_actionable_error(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(auth.shutil, "which", lambda command: None)

    with pytest.raises(auth.CodexCLIError, match="Codex CLI was not found"):
        auth.login_status()
