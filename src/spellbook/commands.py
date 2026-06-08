# ████████╗██╗  ██╗███████╗     ██████╗ ██████╗ ██████╗ ███████╗██╗  ██╗
# ╚══██╔══╝██║  ██║██╔════╝    ██╔════╝██╔═══██╗██╔══██╗██╔════╝╚██╗██╔╝
#    ██║   ███████║█████╗      ██║     ██║   ██║██║  ██║█████╗   ╚███╔╝ 
#    ██║   ██╔══██║██╔══╝      ██║     ██║   ██║██║  ██║██╔══╝   ██╔██╗ 
#    ██║   ██║  ██║███████╗    ╚██████╗╚██████╔╝██████╔╝███████╗██╔╝ ██╗
#    ╚═╝   ╚═╝  ╚═╝╚══════╝     ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝
                                                                      

"""Top-level Spellbook commands."""

from __future__ import annotations

from pathlib import Path

import typer
from rich.pretty import Pretty

from spellbook.console import console, render_banner, render_help_table, render_intro_animation, status_panel
from spellbook.state import load_session, save_session
from spellbook.theme import PALETTE


def help_command() -> None:
    render_intro_animation()
    console.print(render_banner())
    console.print(render_help_table())
    console.print(status_panel("💡 TIP", "Use fantasy commands or plain aliases.", PALETTE["muted"]))


def exit_command() -> None:
    console.print(status_panel("👋 EXIT", "Closing the spellbook.", PALETTE["success"]))
    raise typer.Exit(0)


def goblin_command(problem: str = typer.Argument(..., help="Describe the bug to investigate.")) -> None:
    save_session({"last_command": "goblin", "problem": problem})
    console.print(status_panel("👹 GOBLIN SIGHTED", problem, PALETTE["danger"]))
    console.print("Suggested flow: inspect failing files, reproduce issue, patch bug, verify with /alchemy.")


def alchemy_command(target: str = typer.Argument("", help="Optional test target or note.")) -> None:
    session = load_session()
    focus = target or session.get("problem") or "current quest"
    console.print(status_panel("🧪 ALCHEMY", f"Prepare verification for {focus}.", PALETTE["success"]))
    console.print("Next step: wire this command to pytest, CI checks, or a backend verification runner.")


def coins_command() -> None:
    session = load_session()
    payload = {
        "tokens_used": 0,
        "estimated_cost": "$0.00",
        "last_command": session.get("last_command", "none"),
    }
    console.print(status_panel("🪙 COINS", "Usage ledger is ready for backend integration.", PALETTE["gold"]))
    console.print(Pretty(payload))


def dragons_command(area: str = typer.Argument("", help="Optional deployment or system area.")) -> None:
    target = area or "deployment, CI, or production"
    console.print(status_panel("🐉 DRAGONS", f"Inspect threats in {target}.", PALETTE["danger"]))
    console.print("Future hook: connect this command to deploy logs, CI failures, and incident traces.")


def scroll_command(path: Path = typer.Argument(..., exists=False, help="File or path to explain.")) -> None:
    console.print(status_panel("📜 SCROLL", f"Explain {path}.", PALETTE["accent"]))
    if path.exists():
        console.print(f"Path exists: {path}")
    else:
        console.print("Path not found yet, but the command grammar is in place.")


def map_command() -> None:
    files = sorted(str(path) for path in Path.cwd().iterdir())
    console.print(status_panel("📍 MAP", "Survey the current project roots.", PALETTE["accent"]))
    console.print(Pretty(files))


def quest_command(task: str = typer.Argument(..., help="Describe the implementation task.")) -> None:
    save_session({"last_command": "quest", "task": task})
    console.print(status_panel("🧭 QUEST", task, PALETTE["success"]))
    console.print("Future hook: send this task to your backend runner or agent interface.")
