//! Pixel-art frames for Spellbook.

// Built with concat! because a `\` continuation would eat the leading space.
pub const SPELLBOOK_LOGO: &str = concat!(
    " █████  ██████  ███████ ██      ██      ██████   ██████   ██████  ██   ██\n",
    "██      ██   ██ ██      ██      ██      ██   ██ ██    ██ ██    ██ ██  ██\n",
    " █████  ██████  █████   ██      ██      ██████  ██    ██ ██    ██ █████\n",
    "     ██ ██      ██      ██      ██      ██   ██ ██    ██ ██    ██ ██  ██\n",
    "██████  ██      ███████ ███████ ███████ ██████   ██████   ██████  ██   ██",
);

const LOGO_DITHERED: &str = "\
▓█████  ▓█████▓ ▓██████ ▓▓      ▓▓      ▓█████▓  ▓█████   ▓█████  ▓▓   ▓▓
▓▓      ▓▓   ▓▓ ▓▓      ▓▓      ▓▓      ▓▓   ▓▓ ▓▓    ▓▓ ▓▓    ▓▓ ▓▓  ▓▓
▓█████  ▓█████▓ ▓████   ▓▓      ▓▓      ▓█████▓ ▓▓    ▓▓ ▓▓    ▓▓ ▓████
    ▓▓  ▓▓      ▓▓      ▓▓      ▓▓      ▓▓   ▓▓ ▓▓    ▓▓ ▓▓    ▓▓ ▓▓  ▓▓
▓█████  ▓▓      ▓██████ ▓██████ ▓██████ ▓█████▓  ▓█████   ▓█████  ▓▓   ▓▓";

const LOGO_FADED: &str = "\
▒████▒  ▒████▒ ▒█████▒ ▒▒      ▒▒      ▒████▒  ▒████   ▒████  ▒▒   ▒▒
▒▒      ▒▒  ▒▒ ▒▒      ▒▒      ▒▒      ▒▒  ▒▒ ▒▒  ▒▒ ▒▒  ▒▒ ▒▒  ▒▒
▒████▒  ▒████▒ ▒████   ▒▒      ▒▒      ▒████▒ ▒▒  ▒▒ ▒▒  ▒▒ ▒████
    ▒▒  ▒▒     ▒▒      ▒▒      ▒▒      ▒▒  ▒▒ ▒▒  ▒▒ ▒▒  ▒▒ ▒▒  ▒▒
▒████▒  ▒▒     ▒█████▒ ▒█████▒ ▒█████▒ ▒████▒  ▒████   ▒████  ▒▒   ▒▒";

/// Each frame is a logo plus four status lines, so frames redraw in place.
pub const INTRO_FRAMES: [(&str, [&str; 4]); 3] = [
    (
        SPELLBOOK_LOGO,
        [
            " [📖 APP ] SPELLBOOK",
            " [✨ INFO] Opening the hardware grimoire.",
            " [🔌 BUS ] Warming transport adapters",
            " [🤖 MODE] Summoning device manifests",
        ],
    ),
    (
        LOGO_DITHERED,
        [
            " [📖 APP ] SPELLBOOK",
            " [✨ INFO] Opening the hardware grimoire..",
            " [🔌 BUS ] Warming transport adapters",
            " [🤖 MODE] Summoning device manifests",
        ],
    ),
    (
        LOGO_FADED,
        [
            " [📖 APP ] SPELLBOOK",
            " [✨ INFO] Hardware grimoire open",
            " [🔌 BUS ] Transport adapters ready",
            " [🤖 MODE] Awaiting command_",
        ],
    ),
];

/// Wand frames for long-running work. Every frame has the same width.
pub const MAGIC_WAND_FRAMES: [&str; 8] = [
    "🪄  ·      ",
    " 🪄 ✦     ",
    "  🪄 ✨    ",
    "   🪄 ✦   ",
    "    🪄  · ",
    "   ✨ 🪄  ",
    " ✦  🪄 ✨ ",
    "✨ 🪄  ✦  ",
];
