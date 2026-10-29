// SPDX-License-Identifier: MIT
//! Original host-only layout illustration. No game, engine or device input.
use ntx::{
    touch::{Area, Target},
    viewport::{Geometry, Insets, WindowGeometry},
};
use std::{fmt::Write as _, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("usage: controls-preview <new-output-directory>")?,
    );
    std::fs::create_dir(&root)?;
    for (w, h, density, bottom) in [
        (840., 420., 1., 0.),
        (800., 700., 1., 0.),
        (390., 844., 1., 0.),
        (2076., 2152., 2.4375, 80.),
    ] {
        let bounds = Area {
            left: 0.,
            top: 0.,
            right: w,
            bottom: h,
        };
        let g = Geometry::new(
            WindowGeometry {
                window: bounds,
                surface: bounds,
                content: bounds,
                input_origin: (0., 0.),
                clearance: Insets {
                    bottom,
                    ..Insets::default()
                },
                density,
            },
            w,
            h,
        )
        .ok_or("invalid preview geometry")?;
        let picture = g.picture(1.25);
        let layout = &g.layout;
        let mut svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}"><rect width="{w}" height="{h}" fill="#111418"/><text x="50%" y="28" fill="#eef1eb" font-family="DejaVu Sans" font-size="16" text-anchor="middle">ntx · original control-layout preview</text>"##
        );
        writeln!(
            svg,
            r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#233d47"/><text x="50%" y="{}" fill="#eef1eb" font-family="DejaVu Sans" font-size="{}" text-anchor="middle">Game picture · actual 1.25 aspect fixture</text>"##,
            picture.left,
            picture.top,
            picture.right - picture.left,
            picture.bottom - picture.top,
            (picture.top + picture.bottom) / 2.,
            18. * density
        )?;
        for z in &layout.zones {
            let a = z.area;
            let (x, y) = a.center();
            let color = z.target.color();
            let color = match z.target {
                Target::Stick(ntx::Control::CX, _) => 0xffe6cf51,
                Target::Stick(..) => 0xffaaa6a2,
                Target::Pause => 0xff303b40,
                _ => (color & 0xff00ff00) | ((color & 255) << 16) | ((color >> 16) & 255),
            };
            if matches!(z.target, Target::Stick(..)) {
                let (x, y, r) = ntx::presentation::stick_visual(a, None);
                writeln!(
                    svg,
                    r##"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="#65716b"/><circle cx="{x}" cy="{y}" r="{r}" fill="#292b2e" stroke="#65716b"/><circle cx="{x}" cy="{y}" r="{}" fill="#{:06x}"/>"##,
                    a.left,
                    a.top,
                    a.right - a.left,
                    a.bottom - a.top,
                    r * 0.36,
                    color & 0xffffff
                )?;
                continue;
            }
            let radius = if matches!(z.target, Target::Button(_) | Target::Stick(..)) {
                (a.right - a.left).min(a.bottom - a.top) / 2.
            } else {
                10.
            };
            writeln!(
                svg,
                r##"<rect x="{}" y="{}" width="{}" height="{}" rx="{radius}" fill="#{:06x}" stroke="#65716b" stroke-width="1"/><text x="{x}" y="{}" fill="{}" font-family="DejaVu Sans" font-size="{}" text-anchor="middle">{}</text>"##,
                a.left,
                a.top,
                a.right - a.left,
                a.bottom - a.top,
                color & 0xffffff,
                y + 4. * density,
                if z.target == Target::Pause {
                    "#eef1eb"
                } else {
                    "#13231d"
                },
                12. * density,
                z.label.to_string_lossy()
            )?;
        }
        svg.push_str("</svg>\n");
        std::fs::write(root.join(format!("{}x{}.svg", w as u32, h as u32)), svg)?;
    }
    Ok(())
}
