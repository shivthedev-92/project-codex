from spellbook.art import INTRO_FRAMES, SPELLBOOK_LOGO


def test_intro_frames_are_available() -> None:
    assert len(INTRO_FRAMES) >= 2
    assert all("SPELLBOOK" in frame for frame in INTRO_FRAMES)
    assert all("📖" in frame for frame in INTRO_FRAMES)
    assert any("Awaiting command" in frame for frame in INTRO_FRAMES)


def test_static_logo_is_pixel_art() -> None:
    assert "█████" in SPELLBOOK_LOGO
    assert "SPELLBOOK" not in SPELLBOOK_LOGO


def test_intro_frames_use_pixel_art_blocks() -> None:
    assert not all(frame.isascii() for frame in INTRO_FRAMES)
    assert any("█" in frame for frame in INTRO_FRAMES)
