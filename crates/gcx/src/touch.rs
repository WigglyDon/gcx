// SPDX-License-Identifier: MIT
//! Position based GameCube touch policy; trigger travel never infers a click.
use crate::{Control, Pad};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub struct Area {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
impl Area {
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }
    pub fn center(self) -> (f32, f32) {
        (
            (self.left + self.right) / 2.0,
            (self.top + self.bottom) / 2.0,
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Button(Control),
    Stick(Control, Control),
    Travel(Control),
    Pause,
}
#[derive(Clone, Copy, Debug)]
pub struct Zone {
    pub area: Area,
    pub target: Target,
    pub label: &'static CStrLabel,
}
pub type CStrLabel = std::ffi::CStr;
pub struct Layout {
    pub game: Area,
    pub zones: Vec<Zone>,
}
fn area(x: f32, y: f32, radius: f32) -> Area {
    Area {
        left: x - radius,
        top: y - radius,
        right: x + radius,
        bottom: y + radius,
    }
}
impl Layout {
    pub fn new(width: f32, height: f32) -> Self {
        let width = width.max(1.0);
        let height = height.max(1.0);
        let (game, left, right) = if width > height * 1.25 {
            let side = (width * 0.22).min(height * 0.5);
            (
                Area {
                    left: side,
                    top: 0.0,
                    right: width - side,
                    bottom: height,
                },
                Area {
                    left: 0.0,
                    top: 0.0,
                    right: side,
                    bottom: height,
                },
                Area {
                    left: width - side,
                    top: 0.0,
                    right: width,
                    bottom: height,
                },
            )
        } else {
            let top = height * 0.6;
            (
                Area {
                    left: 0.0,
                    top: 0.0,
                    right: width,
                    bottom: top,
                },
                Area {
                    left: 0.0,
                    top,
                    right: width * 0.5,
                    bottom: height,
                },
                Area {
                    left: width * 0.5,
                    top,
                    right: width,
                    bottom: height,
                },
            )
        };
        let mut zones = Vec::new();
        for (pane, is_left) in [(left, true), (right, false)] {
            let w = pane.right - pane.left;
            let h = pane.bottom - pane.top;
            let r = (w * 0.15).min(h * 0.1).max(1.0);
            let point = |x: f32, y: f32| (pane.left + x * w, pane.top + y * h);
            let (x, y) = point(0.43, 0.46);
            zones.push(Zone {
                area: area(x, y, r * 2.0),
                target: if is_left {
                    Target::Stick(Control::StickX, Control::StickY)
                } else {
                    Target::Stick(Control::CX, Control::CY)
                },
                label: if is_left { c"Control" } else { c"C Stick" },
            });
            zones.push(Zone {
                area: Area {
                    left: pane.left + w * 0.08,
                    top: pane.top + h * 0.04,
                    right: pane.left + w * 0.68,
                    bottom: pane.top + h * 0.15,
                },
                target: Target::Travel(if is_left {
                    Control::LTravel
                } else {
                    Control::RTravel
                }),
                label: if is_left { c"L travel" } else { c"R travel" },
            });
            let (x, y) = point(0.86, 0.095);
            zones.push(Zone {
                area: area(x, y, r),
                target: Target::Button(if is_left {
                    Control::LClick
                } else {
                    Control::RClick
                }),
                label: if is_left { c"L click" } else { c"R click" },
            });
            let buttons = if is_left {
                [
                    (Control::Up, 0.3, 0.75, c"Up"),
                    (Control::Down, 0.3, 0.94, c"Down"),
                    (Control::Left, 0.13, 0.845, c"Left"),
                    (Control::Right, 0.47, 0.845, c"Right"),
                    (Control::Start, 0.8, 0.82, c"Start"),
                ]
            } else {
                [
                    (Control::A, 0.76, 0.7, c"A"),
                    (Control::B, 0.46, 0.8, c"B"),
                    (Control::X, 0.85, 0.46, c"X"),
                    (Control::Y, 0.73, 0.25, c"Y"),
                    (Control::Z, 0.85, 0.94, c"Z"),
                ]
            };
            for (button, x, y, label) in buttons {
                let (x, y) = point(x, y);
                zones.push(Zone {
                    area: area(x, y, r),
                    target: Target::Button(button),
                    label,
                });
            }
        }
        zones.push(Zone {
            area: Area {
                left: left.left + (left.right - left.left) * 0.74,
                top: left.top + (left.bottom - left.top) * 0.48,
                right: left.left + (left.right - left.left) * 0.98,
                bottom: left.top + (left.bottom - left.top) * 0.60,
            },
            target: Target::Pause,
            label: c"Pause",
        });
        Self { game, zones }
    }
}
#[derive(Clone, Copy)]
struct Capture {
    zone: Zone,
    x: f32,
    y: f32,
}
#[derive(Default)]
pub struct TouchPad {
    captures: BTreeMap<i32, Capture>,
}
impl TouchPad {
    pub const fn new() -> Self {
        Self {
            captures: BTreeMap::new(),
        }
    }
    /// A captured pointer keeps its target while it moves. Layout changes cancel captures.
    pub fn down(&mut self, id: i32, x: f32, y: f32, layout: &Layout) -> bool {
        if !x.is_finite() || !y.is_finite() || self.captures.len() >= 16 {
            return false;
        }
        if let Some(zone) = layout
            .zones
            .iter()
            .rev()
            .find(|z| z.area.contains(x, y))
            .copied()
        {
            if matches!(zone.target, Target::Pause) {
                return true;
            }
            if matches!(zone.target, Target::Stick(..) | Target::Travel(_))
                && self
                    .captures
                    .values()
                    .any(|capture| capture.zone.target == zone.target)
            {
                return false;
            }
            self.captures.insert(id, Capture { zone, x, y });
        }
        false
    }
    pub fn moved(&mut self, id: i32, x: f32, y: f32) {
        if x.is_finite()
            && y.is_finite()
            && let Some(c) = self.captures.get_mut(&id)
        {
            c.x = x;
            c.y = y;
        }
    }
    pub fn up(&mut self, id: i32) {
        self.captures.remove(&id);
    }
    pub fn cancel(&mut self) {
        self.captures.clear();
    }
    pub fn pad(&self) -> Pad {
        let mut pad = Pad::default();
        for c in self.captures.values() {
            match c.zone.target {
                Target::Button(control) => {
                    let _ = pad.set(control, 1.0);
                }
                Target::Travel(control) => {
                    let value = ((c.x - c.zone.area.left)
                        / (c.zone.area.right - c.zone.area.left).max(1.0))
                    .clamp(0.0, 1.0);
                    let _ = pad.set(control, f64::from(value));
                }
                Target::Stick(cx, cy) => {
                    let (x, y) = c.zone.area.center();
                    let radius = ((c.zone.area.right - c.zone.area.left) * 0.5).max(1.0);
                    let mut x = (c.x - x) / radius;
                    let mut y = (y - c.y) / radius;
                    let boundary = x.abs().max(y.abs())
                        + (std::f32::consts::SQRT_2 - 1.0) * x.abs().min(y.abs());
                    if boundary > 1.0 {
                        x /= boundary;
                        y /= boundary;
                    }
                    let dead_zone = 0.08;
                    if x.abs().max(y.abs()) < dead_zone {
                        x = 0.0;
                        y = 0.0;
                    }
                    let _ = pad.set(cx, f64::from(x));
                    let _ = pad.set(cy, f64::from(y));
                }
                Target::Pause => {}
            }
        }
        pad
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn releasing_one_pointer_does_not_release_another_or_click_a_partial_trigger() {
        let layout = Layout::new(1800.0, 1000.0);
        let trigger = layout
            .zones
            .iter()
            .find(|z| matches!(z.target, Target::Travel(Control::LTravel)))
            .unwrap();
        let button = layout
            .zones
            .iter()
            .find(|z| matches!(z.target, Target::Button(Control::A)))
            .unwrap();
        let (x, y) = button.area.center();
        let mut touch = TouchPad::default();
        touch.down(1, x, y, &layout);
        touch.down(2, x, y, &layout);
        touch.down(
            3,
            trigger.area.left + (trigger.area.right - trigger.area.left) * 0.4,
            trigger.area.center().1,
            &layout,
        );
        touch.up(1);
        let pad = touch.pad();
        assert_eq!(pad.values()[Control::A as usize], 1.0);
        assert!((pad.values()[Control::LTravel as usize] - 0.4).abs() < 1e-5);
        assert_eq!(pad.values()[Control::LClick as usize], 0.0);
        touch.cancel();
        assert!(touch.pad().values().iter().all(|v| *v == 0.0));
    }
    #[test]
    fn tiny_surfaces_and_offscreen_drags_produce_finite_bounded_values() {
        for (w, h) in [(0.0, 0.0), (1.0, 2.0), (800.0, 1200.0)] {
            let layout = Layout::new(w, h);
            let mut touch = TouchPad::default();
            for (i, zone) in layout.zones.iter().enumerate() {
                let (x, y) = zone.area.center();
                touch.down(i as i32, x, y, &layout);
                touch.moved(i as i32, 1e8, -1e8);
            }
            assert!(
                touch
                    .pad()
                    .values()
                    .iter()
                    .all(|v| v.is_finite() && (-1.0..=1.0).contains(v))
            );
        }
    }
}
