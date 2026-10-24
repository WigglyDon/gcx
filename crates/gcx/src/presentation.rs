// SPDX-License-Identifier: MIT
//! Shared gcx presentation values; no native handles or emulator state.
pub const BACKGROUND: u32 = 0xff111418;
pub const SURFACE: u32 = 0xff22282c;
pub const TEXT: u32 = 0xffeef1eb;
pub const MUTED: u32 = 0xffa9b2b2;
pub const ACCENT: u32 = 0xff85ca72;
pub const INK: u32 = 0xff173322;
pub const DANGER: u32 = 0xffdca7a0;

/// File-provider names remain intact in storage. Present only their title;
/// region/revision/file-container labels are not a game's public identity.
pub fn game_title(name: &str) -> String {
    let mut title = name.trim().to_owned();
    for suffix in [".rvz", ".iso", ".gcm"] {
        if title.to_ascii_lowercase().ends_with(suffix) {
            title.truncate(title.len() - suffix.len());
            break;
        }
    }
    while let Some(start) = title.rfind(" (") {
        if !title.ends_with(')') {
            break;
        }
        let tag = &title[start + 2..title.len() - 1];
        if [
            "USA",
            "USA, Canada",
            "Europe",
            "Japan",
            "Australia",
            "Korea",
            "World",
        ]
        .contains(&tag)
            || tag.starts_with("Rev ")
        {
            title.truncate(start);
        } else {
            break;
        }
    }
    if let Some((name, rest)) = title.split_once(", The - ") {
        format!("The {name} — {rest}")
    } else {
        title
    }
}
/// Adjust decorative alpha only; never filters, buffers or delays logical input.
pub fn opacity(abgr: u32, percent: u8) -> u32 {
    let alpha = (abgr >> 24) * u32::from(percent.clamp(30, 100)) / 100;
    (abgr & 0x00ff_ffff) | (alpha << 24)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn library_names_keep_disc_and_subtitle_but_drop_known_file_metadata() {
        assert_eq!(
            game_title("Legend of Zelda, The - The Wind Waker (USA, Canada).rvz"),
            "The Legend of Zelda — The Wind Waker"
        );
        assert_eq!(
            game_title("Example (Disc 2) (USA) (Rev 1).ISO"),
            "Example (Disc 2)"
        );
        assert_eq!(
            game_title("My game (Director's Cut).gcm"),
            "My game (Director's Cut)"
        );
    }
    #[test]
    fn opacity_preserves_gamecube_color_and_existing_translucency() {
        assert_eq!(opacity(0xff68b65a, 50), 0x7f68b65a);
        assert_eq!(opacity(0x703c3b3a, 50), 0x383c3b3a);
    }
}
