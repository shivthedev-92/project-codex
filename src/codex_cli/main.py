# ████████╗██╗  ██╗███████╗     ██████╗ ██████╗ ██████╗ ███████╗██╗  ██╗
# ╚══██╔══╝██║  ██║██╔════╝    ██╔════╝██╔═══██╗██╔══██╗██╔════╝╚██╗██╔╝
#    ██║   ███████║█████╗      ██║     ██║   ██║██║  ██║█████╗   ╚███╔╝ 
#    ██║   ██╔══██║██╔══╝      ██║     ██║   ██║██║  ██║██╔══╝   ██╔██╗ 
#    ██║   ██║  ██║███████╗    ╚██████╗╚██████╔╝██████╔╝███████╗██╔╝ ██╗
#    ╚═╝   ╚═╝  ╚═╝╚══════╝     ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝

"""CLI entrypoint."""

from __future__ import annotations

import sys

import typer

from codex_cli.commands import (
    alchemy_command,
    coins_command,
    dragons_command,
    goblin_command,
    help_command,
    map_command,
    quest_command,
    scroll_command,
)
from codex_cli.theme import ALIASES

app = typer.Typer(
    add_completion=False,
    rich_markup_mode="rich",
    help="A retro fantasy coding CLI.",
    no_args_is_help=True,
)

app.command("help")(help_command)
app.command("goblin")(goblin_command)
app.command("alchemy")(alchemy_command)
app.command("coins")(coins_command)
app.command("dragons")(dragons_command)
app.command("scroll")(scroll_command)
app.command("map")(map_command)
app.command("quest")(quest_command)


def normalize_argv() -> None:
    if len(sys.argv) < 2:
        return
    raw = sys.argv[1].lstrip("/")
    command = ALIASES.get(raw)
    if not command:
        return
    sys.argv[1] = command


def run() -> None:
    normalize_argv()
    app()


if __name__ == "__main__":
    run()
