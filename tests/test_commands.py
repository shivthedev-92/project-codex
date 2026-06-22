from __future__ import annotations

from contextlib import nullcontext

from rich.markdown import Markdown

from spellbook import commands


def test_quest_sends_task_to_codex(monkeypatch) -> None:
    output: list[object] = []

    monkeypatch.setattr(commands, "save_session", lambda payload: None)
    monkeypatch.setattr(commands, "magic_wand", lambda message: nullcontext())
    monkeypatch.setattr(commands, "run_agent", lambda task: "Codex response")
    monkeypatch.setattr(commands.console, "print", lambda value: output.append(value))

    commands.quest_command("Explain this project")

    answers = [value for value in output if isinstance(value, Markdown)]
    assert len(answers) == 1
    assert answers[0].markup == "Codex response"
