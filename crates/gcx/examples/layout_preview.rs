// SPDX-License-Identifier: MIT
//! Off-device preview of the actual shared geometry. No game artwork or pixels.
use gcx::{
    Control,
    touch::{Area, Target},
    viewport::{Geometry, Insets, WindowGeometry},
};
fn main() {
    let bounds = Area {
        left: 0.0,
        top: 0.0,
        right: 830.0,
        bottom: 393.0,
    };
    let geometry = Geometry::new(
        WindowGeometry {
            window: bounds,
            surface: bounds,
            content: bounds,
            input_origin: (0.0, 0.0),
            clearance: Insets {
                top: 24.0,
                bottom: 24.0,
                ..Insets::default()
            },
            density: 1.0,
        },
        830.0,
        393.0,
    )
    .unwrap();
    let aspect = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(1.25);
    let picture = geometry.picture(aspect);
    let layout = geometry.layout;
    println!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1660" height="786" viewBox="0 0 830 393"><rect width="830" height="393" fill="#101214"/>"##
    );
    let a = picture;
    let w = a.right - a.left;
    let h = a.bottom - a.top;
    println!(
        r##"<rect x="{}" y="{}" width="{w}" height="{h}" fill="#26323a"/><text x="415" y="{}" text-anchor="middle" fill="#b9c5cd" font-family="DejaVu Sans" font-size="16">Game picture · aspect {aspect}</text>"##,
        a.left,
        a.top,
        a.top + h / 2.0
    );
    for zone in layout.zones {
        let a = zone.area;
        let (x, y) = a.center();
        let w = a.right - a.left;
        let h = a.bottom - a.top;
        let abgr = zone.target.color();
        let color = format!(
            "#{:02x}{:02x}{:02x}",
            abgr & 255,
            (abgr >> 8) & 255,
            (abgr >> 16) & 255
        );
        let rect = |x, y, w, h, fill: &str| {
            println!(r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="5" fill="{fill}"/>"##)
        };
        let circle =
            |r, fill: &str| println!(r##"<circle cx="{x}" cy="{y}" r="{r}" fill="{fill}"/>"##);
        let mut label = zone.label.to_str().unwrap();
        let mut font = (h * 0.4)
            .min(w / (label.len() as f32 * 0.65 + 1.0))
            .clamp(10.0, 32.0);
        let mut ink = "#252629";
        match zone.target {
            Target::Stick(cx, _) => {
                circle(w * 0.5, "#292a2c");
                circle(
                    w * 0.5 * if cx == Control::CX { 0.38 } else { 0.48 },
                    if cx == Control::CX {
                        "#e6cf51"
                    } else {
                        "#aeb3b5"
                    },
                );
                if cx != Control::CX {
                    continue;
                }
                label = "C";
                font = w * 0.2;
            }
            Target::Dpad => {
                rect(a.left + w / 3.0, a.top, w / 3.0, h, &color);
                rect(a.left, a.top + h / 3.0, w, h / 3.0, &color);
                continue;
            }
            Target::Button(_) if (w - h).abs() < 1.0 => circle(w * 0.5, &color),
            Target::Button(_) => rect(a.left, a.top, w, h, &color),
            _ => {
                rect(a.left, a.top, w, h, &color);
                ink = "#eeeeee";
            }
        }
        println!(
            r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="central" fill="{ink}" font-family="DejaVu Sans" font-size="{font}">{label}</text>"##
        );
    }
    println!("</svg>");
}
