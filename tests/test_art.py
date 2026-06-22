from spellbook.art import INTRO_FRAMES, MAGIC_WAND_FRAMES, SPELLBOOK_LOGO


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


def test_magic_wand_frames_create_motion_and_sparkles() -> None:
    assert len(MAGIC_WAND_FRAMES) >= 6
    assert all("🪄" in frame for frame in MAGIC_WAND_FRAMES)
    assert any("✨" in frame for frame in MAGIC_WAND_FRAMES)
    assert len(set(MAGIC_WAND_FRAMES)) == len(MAGIC_WAND_FRAMES)
