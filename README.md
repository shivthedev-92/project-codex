# Spellbook

A retro fantasy command-line spellbook for coding workflows.

This project explores a more accessible, concept-oriented tool inspired by an
arcane spellbook aesthetic. Common development tasks are exposed as memorable
commands like `goblin`, `alchemy`, and `coins`.

![Spellbook terminal screenshot](./docs/spellbook-screenshot.svg)

## Vision

- Intuitive, human-first commands
- Light fantasy theme without sacrificing clarity
- Python-based interface layer
- Easy to package into a standalone executable later

## Planned Commands

- `spellbook` or `help`: show command help
- `/goblin` or `fix`: investigate a bug
- `/alchemy` or `test`: verify or test a fix
- `/coins` or `usage`: show usage information
- `/dragons` or `doctor`: inspect deployment or production failures
- `/scroll` or `explain`: explain code or files
- `/map` or `repo`: orient around the repository
- `/quest` or `forge`: start a new implementation task

## Quick Start

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -e .
spellbook
spellbook /goblin "login breaks on mobile"
spellbook /alchemy
```

Connect Spellbook to your ChatGPT account through Codex's browser login:

```bash
spellbook /settings status
spellbook /settings login
spellbook /settings logout
```

The login command opens the official ChatGPT sign-in page. Codex handles the
localhost callback and credential storage; Spellbook does not read or store
your access token.

## Bash Alias

Install a permanent `spellbook` alias without activating the virtual
environment for every terminal session:

```bash
cd /home/shivthedev/Documents/projects/project-codex
./scripts/install-bash-alias
source ~/.bashrc
```

You can then run Spellbook from any directory:

```bash
spellbook
spellbook /settings status
spellbook /scroll src/spellbook/main.py
spellbook /quest "describe the implementation task"
```

`/quest` runs the task through the authenticated Codex CLI in the current
directory. Codex receives workspace-write access so implementation quests can
edit project files; review the requested task before casting it.

Running `spellbook` without a command opens an interactive session. Enter
commands at the prompt and use `/exit` (or Ctrl+D) to close it:

```text
spellbook> /settings status
spellbook> /scroll README.md
spellbook> /exit
```

The alias points to `scripts/spellbook`, which executes the project's
`.venv/bin/spellbook` directly. Moving the repository requires reinstalling the
alias from its new location.

You can also use plain aliases:

```bash
spellbook help
spellbook fix "broken login flow"
spellbook test
```

## Project Layout

```text
src/spellbook/
  main.py
  console.py
  theme.py
  state.py
  commands.py
tests/
```

## Notes

This scaffold is intentionally thin. It focuses on:

- command grammar
- visual tone
- session/config foundations
- future compatibility with a backend agent or API layer

Long-running model calls should use the terminal-safe processing animation:

```python
from spellbook.console import magic_wand

with magic_wand("Reading the scroll"):
    response = model_runner.run(prompt)
```

The wand and sparkles animate in an interactive terminal. Redirected output,
CI, and terminals without color receive a single stable status line.

## Contributing

Spellbook is early and intentionally approachable. Good first contributions are:

- Make existing commands functional, starting with `/alchemy`, `/map`, and `/scroll`
- Add tests for command aliases and terminal rendering helpers
- Improve terminal layout across different terminal widths
- Add new themed commands, such as `🏰 /castle` for model/provider information
- Keep the fantasy theme playful without making command behavior unclear

To work locally:

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
pytest
spellbook
```

Please keep changes small, tested, and easy to understand.
