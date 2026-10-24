// SPDX-License-Identifier: MIT
//! Original host-only layout illustration. No game, engine or device input.
use gcx::touch::{Layout, Target};
use std::{fmt::Write as _, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("usage: controls-preview <new-output-directory>")?,
    );
    std::fs::create_dir(&root)?;
    for (w, h) in [(840., 420.), (800., 700.), (390., 844.)] {
        let layout = Layout::new(w, h);
        let mut svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}"><rect width="{w}" height="{h}" fill="#111418"/><text x="50%" y="28" fill="#eef1eb" font-family="DejaVu Sans" font-size="16" text-anchor="middle">gcx · original control-layout preview</text>"##
        );
        for z in layout.zones {
            let a = z.area;
            let (x, y) = a.center();
            let color = z.target.color();
            let color = match z.target {
                Target::Stick(gcx::Control::CX, _) => 0xffe6cf51,
                Target::Stick(..) => 0xffaaa6a2,
                Target::Pause => 0xff303b40,
                _ => (color & 0xff00ff00) | ((color & 255) << 16) | ((color >> 16) & 255),
            };
            let radius = if matches!(z.target, Target::Button(_) | Target::Stick(..)) {
                (a.right - a.left).min(a.bottom - a.top) / 2.
            } else {
                10.
            };
            writeln!(
                svg,
                r##"<rect x="{}" y="{}" width="{}" height="{}" rx="{radius}" fill="#{:06x}" stroke="#65716b" stroke-width="1"/><text x="{x}" y="{}" fill="{}" font-family="DejaVu Sans" font-size="12" text-anchor="middle">{}</text>"##,
                a.left,
                a.top,
                a.right - a.left,
                a.bottom - a.top,
                color & 0xffffff,
                y + 4.,
                if z.target == Target::Pause {
                    "#eef1eb"
                } else {
                    "#13231d"
                },
                z.label.to_string_lossy()
            )?;
        }
        svg.push_str("</svg>\n");
        std::fs::write(root.join(format!("{}x{}.svg", w as u32, h as u32)), svg)?;
    }
    Ok(())
}
