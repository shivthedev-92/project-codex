from __future__ import annotations

from io import StringIO

import pytest
from rich.console import Console

from spellbook.console import magic_wand


def test_magic_wand_uses_stable_text_when_output_is_not_interactive() -> None:
    stream = StringIO()
    output = Console(file=stream, force_terminal=False, color_system=None)

    with magic_wand("Reading the scroll", target_console=output):
        pass

    assert stream.getvalue() == "🪄 Reading the scroll…\n"


def test_magic_wand_does_not_swallow_model_errors() -> None:
    output = Console(file=StringIO(), force_terminal=False, color_system=None)

    with pytest.raises(RuntimeError, match="model failed"):
        with magic_wand(target_console=output):
            raise RuntimeError("model failed")
