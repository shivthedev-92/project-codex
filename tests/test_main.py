from __future__ import annotations

import typer

from spellbook import main


def test_normalize_command_supports_slash_and_plain_aliases() -> None:
    assert main.normalize_command(["/scroll", "README.md"]) == ["scroll", "README.md"]
    assert main.normalize_command(["explain", "README.md"]) == ["scroll", "README.md"]
    assert main.normalize_command(["unknown"]) == ["unknown"]


def test_interactive_loop_dispatches_until_exit(monkeypatch) -> None:
    inputs = iter(["", '/quest "fix the map"', "/exit"])
    dispatched: list[list[str]] = []

    monkeypatch.setattr(main, "help_command", lambda: None)
    monkeypatch.setattr(main.console, "input", lambda prompt: next(inputs))
    monkeypatch.setattr(main, "exit_command", lambda: (_ for _ in ()).throw(typer.Exit(0)))

    def fake_app(*, args, prog_name, standalone_mode):
        dispatched.append(args)

    monkeypatch.setattr(main, "app", fake_app)

    main.interactive_loop()

    assert dispatched == [["quest", "fix the map"]]
