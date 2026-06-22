# ████████╗██╗  ██╗███████╗     ██████╗ ██████╗ ██████╗ ███████╗██╗  ██╗
# ╚══██╔══╝██║  ██║██╔════╝    ██╔════╝██╔═══██╗██╔══██╗██╔════╝╚██╗██╔╝
#    ██║   ███████║█████╗      ██║     ██║   ██║██║  ██║█████╗   ╚███╔╝ 
#    ██║   ██╔══██║██╔══╝      ██║     ██║   ██║██║  ██║██╔══╝   ██╔██╗ 
#    ██║   ██║  ██║███████╗    ╚██████╗╚██████╔╝██████╔╝███████╗██╔╝ ██╗
#    ╚═╝   ╚═╝  ╚═╝╚══════╝     ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝

"""Terminal presentation helpers."""

from collections.abc import Iterator
from contextlib import contextmanager
import os
from itertools import cycle
from threading import Event, Thread
from time import sleep

from rich.console import Console
from rich.live import Live
from rich.panel import Panel
from rich.table import Table
from rich.text import Text

from spellbook.art import INTRO_FRAMES, MAGIC_WAND_FRAMES, SPELLBOOK_LOGO
from spellbook.theme import COMMAND_DESCRIPTIONS, PALETTE, SUBTITLE

console = Console()


def _supports_animation(target_console: Console) -> bool:
    """Return whether replacing a live terminal line is safe."""
    return (
        target_console.is_terminal
        and not target_console.no_color
        and os.environ.get("TERM", "").lower() != "dumb"
    )


@contextmanager
def magic_wand(
    message: str = "Consulting the grimoire",
    *,
    interval: float = 0.12,
    target_console: Console | None = None,
) -> Iterator[None]:
    """Animate a wand while a blocking model operation is in progress.

    The worker only redraws presentation; the caller retains control of the
    model call and any exception it raises. Non-interactive output receives one
    stable status line instead of terminal control sequences.
    """
    output = target_console or console
    if not _supports_animation(output):
        output.print(f"🪄 {message}…")
        yield
        return

    stopped = Event()
    live = Live(console=output, refresh_per_second=20, transient=True)

    def animate() -> None:
        for frame in cycle(MAGIC_WAND_FRAMES):
            if stopped.is_set():
                return
            text = Text.assemble(
                (frame, PALETTE["gold"]),
                (message, PALETTE["accent"]),
                ("…", PALETTE["muted"]),
            )
            live.update(text, refresh=True)
            if stopped.wait(interval):
                return

    worker = Thread(target=animate, name="spellbook-magic-wand", daemon=True)
    with live:
        worker.start()
        try:
            yield
        finally:
            stopped.set()
            worker.join(timeout=max(1.0, interval * 2))


def render_intro_animation(cycles: int = 2, delay: float = 0.12) -> None:
    if not console.is_terminal:
        return

    with Live(console=console, refresh_per_second=12, transient=True) as live:
        for _ in range(cycles):
            for frame in INTRO_FRAMES:
                live.update(Panel(frame, border_style=PALETTE["panel"], expand=False))
                sleep(delay)


def render_banner() -> Panel:
    title = Text(SPELLBOOK_LOGO, style=PALETTE["accent"], justify="center")
    subtitle = Text(SUBTITLE, style=PALETTE["gold"], justify="center")
    body = Text.assemble(title, "\n\n", subtitle, "\n", ("✨ STATUS: READY", PALETTE["success"]))
    return Panel(body, border_style=PALETTE["panel"], expand=False)


def render_help_table() -> Table:
    table = Table(title="Command Grimoire", border_style=PALETTE["panel"])
    table.add_column("Command", style=PALETTE["accent"])
    table.add_column("Plain Alias", style=PALETTE["gold"])
    table.add_column("Meaning", style=PALETTE["label"])
    table.add_row("📖 spellbook", "help", COMMAND_DESCRIPTIONS["help"])
    table.add_row("👋 /exit", "quit, bye", COMMAND_DESCRIPTIONS["exit"])
    table.add_row("👹 /goblin", "fix", COMMAND_DESCRIPTIONS["goblin"])
    table.add_row("🧪 /alchemy", "test", COMMAND_DESCRIPTIONS["alchemy"])
    table.add_row("🪙 /coins", "usage", COMMAND_DESCRIPTIONS["coins"])
    table.add_row("🐉 /dragons", "doctor", COMMAND_DESCRIPTIONS["dragons"])
    table.add_row("📜 /scroll", "explain", COMMAND_DESCRIPTIONS["scroll"])
    table.add_row("📍 /map", "repo", COMMAND_DESCRIPTIONS["map"])
    table.add_row("🧭 /quest", "forge", COMMAND_DESCRIPTIONS["quest"])
    table.add_row("⚙️ /settings", "config", COMMAND_DESCRIPTIONS["settings"])
    return table


def status_panel(label: str, message: str, style: str) -> Panel:
    content = Text()
    content.append(f"{label}: ", style=PALETTE["label"])
    content.append(message, style=style)
    return Panel(content, border_style=PALETTE["panel"])
