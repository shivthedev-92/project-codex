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
