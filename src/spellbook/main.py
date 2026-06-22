# ████████╗██╗  ██╗███████╗     ██████╗ ██████╗ ██████╗ ███████╗██╗  ██╗
# ╚══██╔══╝██║  ██║██╔════╝    ██╔════╝██╔═══██╗██╔══██╗██╔════╝╚██╗██╔╝
#    ██║   ███████║█████╗      ██║     ██║   ██║██║  ██║█████╗   ╚███╔╝ 
#    ██║   ██╔══██║██╔══╝      ██║     ██║   ██║██║  ██║██╔══╝   ██╔██╗ 
#    ██║   ██║  ██║███████╗    ╚██████╗╚██████╔╝██████╔╝███████╗██╔╝ ██╗
#    ╚═╝   ╚═╝  ╚═╝╚══════╝     ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝

"""Spellbook entrypoint."""

from __future__ import annotations

import shlex
import sys

import typer
from typer._click.exceptions import ClickException

from spellbook.commands import (
    alchemy_command,
    coins_command,
    dragons_command,
    exit_command,
    goblin_command,
    help_command,
    map_command,
    quest_command,
    scroll_command,
    settings_command,
)
from spellbook.console import console
from spellbook.theme import ALIASES

app = typer.Typer(
    add_completion=False,
    rich_markup_mode="rich",
    help="A retro fantasy coding spellbook.",
    no_args_is_help=True,
)

app.command("help")(help_command)
app.command("exit")(exit_command)
app.command("goblin")(goblin_command)
app.command("alchemy")(alchemy_command)
app.command("coins")(coins_command)
app.command("dragons")(dragons_command)
app.command("scroll")(scroll_command)
app.command("map")(map_command)
app.command("quest")(quest_command)
app.command("settings")(settings_command)


def normalize_command(arguments: list[str]) -> list[str]:
    """Normalize fantasy commands and plain aliases for Typer."""
    if not arguments:
        return arguments
    raw = arguments[0].lstrip("/")
    command = ALIASES.get(raw)
    if not command:
        return arguments
    return [command, *arguments[1:]]


def interactive_loop() -> None:
    """Keep Spellbook open and dispatch commands until the user exits."""
    help_command()

    while True:
        try:
            line = console.input("\n[bold bright_cyan]spellbook>[/] ").strip()
        except (EOFError, KeyboardInterrupt):
            console.print()
            try:
                exit_command()
            except typer.Exit:
                return

        if not line:
            continue

        try:
            arguments = normalize_command(shlex.split(line))
        except ValueError as error:
            console.print(f"[bold red]Invalid command:[/] {error}")
            continue

        if arguments[0] == "exit":
            try:
                exit_command()
            except typer.Exit:
                return

        try:
            app(args=arguments, prog_name="spellbook", standalone_mode=False)
        except typer.Exit:
            return
        except ClickException as error:
            console.print(f"[bold red]{error.format_message()}[/]")


def run() -> None:
    if len(sys.argv) == 1:
        interactive_loop()
        return
    sys.argv[1:] = normalize_command(sys.argv[1:])
    app()


if __name__ == "__main__":
    run()
