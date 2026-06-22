"""Pixel-art frames for Spellbook."""

from __future__ import annotations

SPELLBOOK_LOGO = r"""
 █████  ██████  ███████ ██      ██      ██████   ██████   ██████  ██   ██
██      ██   ██ ██      ██      ██      ██   ██ ██    ██ ██    ██ ██  ██
 █████  ██████  █████   ██      ██      ██████  ██    ██ ██    ██ █████
     ██ ██      ██      ██      ██      ██   ██ ██    ██ ██    ██ ██  ██
██████  ██      ███████ ███████ ███████ ██████   ██████   ██████  ██   ██
""".strip("\n")

INTRO_FRAMES = [
    f"""
{SPELLBOOK_LOGO}

 [📖 APP ] SPELLBOOK
 [✨ INFO] Initializing Spellbook.
 [💾 MEM ] 2048 KB reserved for incantations
 [🔮 MODE] Arcane command layer loading
""".strip("\n"),
    r"""
▓█████  ▓█████▓ ▓██████ ▓▓      ▓▓      ▓█████▓  ▓█████   ▓█████  ▓▓   ▓▓
▓▓      ▓▓   ▓▓ ▓▓      ▓▓      ▓▓      ▓▓   ▓▓ ▓▓    ▓▓ ▓▓    ▓▓ ▓▓  ▓▓
▓█████  ▓█████▓ ▓████   ▓▓      ▓▓      ▓█████▓ ▓▓    ▓▓ ▓▓    ▓▓ ▓████
    ▓▓  ▓▓      ▓▓      ▓▓      ▓▓      ▓▓   ▓▓ ▓▓    ▓▓ ▓▓    ▓▓ ▓▓  ▓▓
▓█████  ▓▓      ▓██████ ▓██████ ▓██████ ▓█████▓  ▓█████   ▓█████  ▓▓   ▓▓

 [📖 APP ] SPELLBOOK
 [✨ INFO] Initializing Spellbook..
 [💾 MEM ] 2048 KB reserved for incantations
 [🔮 MODE] Arcane command layer loading
""".strip("\n"),
    r"""
▒████▒  ▒████▒ ▒█████▒ ▒▒      ▒▒      ▒████▒  ▒████   ▒████  ▒▒   ▒▒
▒▒      ▒▒  ▒▒ ▒▒      ▒▒      ▒▒      ▒▒  ▒▒ ▒▒  ▒▒ ▒▒  ▒▒ ▒▒  ▒▒
▒████▒  ▒████▒ ▒████   ▒▒      ▒▒      ▒████▒ ▒▒  ▒▒ ▒▒  ▒▒ ▒████
    ▒▒  ▒▒     ▒▒      ▒▒      ▒▒      ▒▒  ▒▒ ▒▒  ▒▒ ▒▒  ▒▒ ▒▒  ▒▒
▒████▒  ▒▒     ▒█████▒ ▒█████▒ ▒█████▒ ▒████▒  ▒████   ▒████  ▒▒   ▒▒

 [📖 APP ] SPELLBOOK
 [✨ INFO] Spellbook initialized
 [💾 MEM ] 2048 KB reserved for incantations
 [🔮 MODE] Awaiting command_
""".strip("\n"),
]

# Each frame has the same visual structure so Rich can redraw it in place
# without leaving fragments behind. Emoji glyph widths are normalized by Rich
# for the active terminal.
MAGIC_WAND_FRAMES = (
    "🪄  ·      ",
    " 🪄 ✦     ",
    "  🪄 ✨    ",
    "   🪄 ✦   ",
    "    🪄  · ",
    "   ✨ 🪄  ",
    " ✦  🪄 ✨ ",
    "✨ 🪄  ✦  ",
)
