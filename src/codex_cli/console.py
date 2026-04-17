# ████████╗██╗  ██╗███████╗     ██████╗ ██████╗ ██████╗ ███████╗██╗  ██╗
# ╚══██╔══╝██║  ██║██╔════╝    ██╔════╝██╔═══██╗██╔══██╗██╔════╝╚██╗██╔╝
#    ██║   ███████║█████╗      ██║     ██║   ██║██║  ██║█████╗   ╚███╔╝ 
#    ██║   ██╔══██║██╔══╝      ██║     ██║   ██║██║  ██║██╔══╝   ██╔██╗ 
#    ██║   ██║  ██║███████╗    ╚██████╗╚██████╔╝██████╔╝███████╗██╔╝ ██╗
#    ╚═╝   ╚═╝  ╚═╝╚══════╝     ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝

"""Terminal presentation helpers."""

from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.text import Text

from codex_cli.theme import COMMAND_DESCRIPTIONS, PALETTE, SUBTITLE, TITLE

console = Console()


def render_banner() -> Panel:
    title = Text(TITLE, style=PALETTE["accent"], justify="center")
    subtitle = Text(SUBTITLE, style=PALETTE["gold"], justify="center")
    body = Text.assemble(title, "\n", subtitle, "\n", ("STATUS: READY", PALETTE["success"]))
    return Panel(body, border_style=PALETTE["panel"], expand=False)


def render_help_table() -> Table:
    table = Table(title="Command Grimoire", border_style=PALETTE["panel"])
    table.add_column("Command", style=PALETTE["accent"])
    table.add_column("Plain Alias", style=PALETTE["gold"])
    table.add_column("Meaning", style=PALETTE["label"])
    table.add_row("/codex-open", "help", COMMAND_DESCRIPTIONS["help"])
    table.add_row("/goblin", "fix", COMMAND_DESCRIPTIONS["goblin"])
    table.add_row("/alchemy", "test", COMMAND_DESCRIPTIONS["alchemy"])
    table.add_row("/coins", "usage", COMMAND_DESCRIPTIONS["coins"])
    table.add_row("/dragons", "doctor", COMMAND_DESCRIPTIONS["dragons"])
    table.add_row("/scroll", "explain", COMMAND_DESCRIPTIONS["scroll"])
    table.add_row("/map", "repo", COMMAND_DESCRIPTIONS["map"])
    table.add_row("/quest", "forge", COMMAND_DESCRIPTIONS["quest"])
    return table


def status_panel(label: str, message: str, style: str) -> Panel:
    content = Text()
    content.append(f"{label}: ", style=PALETTE["label"])
    content.append(message, style=style)
    return Panel(content, border_style=PALETTE["panel"])
