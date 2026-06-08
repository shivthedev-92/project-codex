# ████████╗██╗  ██╗███████╗     ██████╗ ██████╗ ██████╗ ███████╗██╗  ██╗
# ╚══██╔══╝██║  ██║██╔════╝    ██╔════╝██╔═══██╗██╔══██╗██╔════╝╚██╗██╔╝
#    ██║   ███████║█████╗      ██║     ██║   ██║██║  ██║█████╗   ╚███╔╝ 
#    ██║   ██╔══██║██╔══╝      ██║     ██║   ██║██║  ██║██╔══╝   ██╔██╗ 
#    ██║   ██║  ██║███████╗    ╚██████╗╚██████╔╝██████╔╝███████╗██╔╝ ██╗
#    ╚═╝   ╚═╝  ╚═╝╚══════╝     ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝

"""Theme constants for Spellbook."""

TITLE = "SPELLBOOK"
SUBTITLE = "Arcane Command Interface"

PALETTE = {
    "accent": "bold bright_cyan",
    "gold": "bold yellow",
    "danger": "bold red",
    "success": "bold green",
    "muted": "grey70",
    "panel": "bright_blue",
    "label": "bold white",
}

ALIASES = {
    "spellbook": "help",
    "help": "help",
    "exit": "exit",
    "quit": "exit",
    "bye": "exit",
    "goblin": "goblin",
    "fix": "goblin",
    "alchemy": "alchemy",
    "test": "alchemy",
    "coins": "coins",
    "usage": "coins",
    "dragons": "dragons",
    "doctor": "dragons",
    "scroll": "scroll",
    "explain": "scroll",
    "map": "map",
    "repo": "map",
    "quest": "quest",
    "forge": "quest",
}

COMMAND_DESCRIPTIONS = {
    "help": "Open the spellbook and show commands.",
    "exit": "Close the spellbook cleanly.",
    "goblin": "Track down a bug or broken flow.",
    "alchemy": "Verify a fix with tests or checks.",
    "coins": "Show token and usage information.",
    "dragons": "Inspect deployment, CI, or production trouble.",
    "scroll": "Explain a file, function, or code path.",
    "map": "Show project structure and likely hotspots.",
    "quest": "Start a new implementation task.",
}
