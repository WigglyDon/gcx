// SPDX-License-Identifier: MIT
//! One presentation and input coordinate owner. Platform measurements are in
//! screen pixels; native input is in window pixels and drawing in buffer pixels.
use crate::{
    Pad,
    touch::{Area, Layout, TouchPad},
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowGeometry {
    /// Actual window bounds, including any screen-space origin.
    pub window: Area,
    /// NativeActivity takes the decor's surface, not its inset content view.
    pub surface: Area,
    /// Framework content bounds, which may already exclude bars/cutouts.
    pub content: Area,
    /// Screen position of native input's window-local (0,0).
    pub input_origin: (f32, f32),
    /// Maximum of stable bars, cutout and mandatory gesture clearance. Keeping
    /// stable bar clearance also protects controls during transient bar reveal.
    pub clearance: Insets,
    pub density: f32,
}

fn valid(a: Area) -> bool {
    [a.left, a.top, a.right, a.bottom]
        .iter()
        .all(|n| n.is_finite())
        && a.right > a.left
        && a.bottom > a.top
}
fn intersection(a: Area, b: Area) -> Area {
    Area {
        left: a.left.max(b.left),
        top: a.top.max(b.top),
        right: a.right.min(b.right),
        bottom: a.bottom.min(b.bottom),
    }
}

#[derive(Clone, Debug)]
pub struct Geometry {
    pub layout: Layout,
    pub density: f32,
    window: WindowGeometry,
    buffer: (f32, f32),
}
impl Geometry {
    pub fn new(window: WindowGeometry, width: f32, height: f32) -> Option<Self> {
        let i = window.clearance;
        if !valid(window.window)
            || !valid(window.surface)
            || !valid(window.content)
            || ![i.left, i.top, i.right, i.bottom]
                .iter()
                .all(|n| n.is_finite() && *n >= 0.0)
            || !window.input_origin.0.is_finite()
            || !window.input_origin.1.is_finite()
            || !window.density.is_finite()
            || window.density <= 0.0
            || !width.is_finite()
            || !height.is_finite()
            || width < 2.0
            || height < 2.0
        {
            return None;
        }
        // This player never requests an independently stretched surface. During
        // rotation the old Java bounds and new buffer can briefly disagree;
        // wait for a coherent measurement instead of admitting shifted targets.
        let surface_aspect = (window.surface.right - window.surface.left)
            / (window.surface.bottom - window.surface.top);
        if ((width / height) / surface_aspect - 1.0).abs() > 0.02 {
            return None;
        }
        let safe = intersection(
            intersection(window.surface, window.content),
            Area {
                left: window.window.left + i.left,
                top: window.window.top + i.top,
                right: window.window.right - i.right,
                bottom: window.window.bottom - i.bottom,
            },
        );
        if !valid(safe) || safe.right - safe.left < 2.0 || safe.bottom - safe.top < 2.0 {
            return None;
        }
        // Intersect rectangles in one space before transforming: an already
        // inset surface/content is never charged the same inset twice.
        let mut layout = Layout::for_display(
            safe.right - safe.left,
            safe.bottom - safe.top,
            window.density,
        );
        let sx = width / (window.surface.right - window.surface.left);
        let sy = height / (window.surface.bottom - window.surface.top);
        let map = |a: &mut Area| {
            a.left = (a.left + safe.left - window.surface.left) * sx;
            a.right = (a.right + safe.left - window.surface.left) * sx;
            a.top = (a.top + safe.top - window.surface.top) * sy;
            a.bottom = (a.bottom + safe.top - window.surface.top) * sy;
        };
        map(&mut layout.game);
        for zone in &mut layout.zones {
            map(&mut zone.area);
        }
        Some(Self {
            layout,
            density: window.density * sx.min(sy),
            window,
            buffer: (width, height),
        })
    }
    pub fn input_to_buffer(&self, x: f32, y: f32) -> (f32, f32) {
        (
            (x + self.window.input_origin.0 - self.window.surface.left) * self.buffer.0
                / (self.window.surface.right - self.window.surface.left),
            (y + self.window.input_origin.1 - self.window.surface.top) * self.buffer.1
                / (self.window.surface.bottom - self.window.surface.top),
        )
    }
    pub fn picture(&self, aspect: f32) -> Area {
        let a = self.layout.game;
        let aspect = if aspect.is_finite() && aspect > 0.0 {
            aspect
        } else {
            1.0
        };
        let width = (a.right - a.left).min((a.bottom - a.top) * aspect);
        let left = a.left + (a.right - a.left - width) * 0.5;
        Area {
            left,
            top: a.top,
            right: left + width,
            bottom: a.top + width / aspect,
        }
    }
}

/// Touch only addresses the geometry actually presented. Any new measurement,
/// invalidation or drawable size change releases captures before accepting input.
pub struct PlayerView {
    window: Option<WindowGeometry>,
    presented: Option<Geometry>,
    touch: TouchPad,
}
impl PlayerView {
    pub const fn new() -> Self {
        Self {
            window: None,
            presented: None,
            touch: TouchPad::new(),
        }
    }
    pub fn measure(&mut self, window: Option<WindowGeometry>) {
        if self.window != window {
            self.invalidate();
            self.window = window;
        }
    }
    pub fn invalidate(&mut self) {
        self.presented = None;
        self.touch.cancel();
    }
    pub fn cancel(&mut self) {
        self.touch.cancel();
    }
    pub fn present(&mut self, width: f32, height: f32) -> Option<&Geometry> {
        if self
            .presented
            .as_ref()
            .is_none_or(|g| g.buffer != (width, height))
        {
            self.invalidate();
            self.presented = self.window.and_then(|w| Geometry::new(w, width, height));
        }
        self.presented.as_ref()
    }
    pub fn down(&mut self, id: i32, x: f32, y: f32) -> bool {
        let Some(g) = &self.presented else {
            return false;
        };
        let (x, y) = g.input_to_buffer(x, y);
        self.touch.down(id, x, y, &g.layout)
    }
    pub fn moved(&mut self, id: i32, x: f32, y: f32) {
        if let Some(g) = &self.presented {
            let (x, y) = g.input_to_buffer(x, y);
            self.touch.moved(id, x, y);
        }
    }
    pub fn up(&mut self, id: i32) {
        self.touch.up(id);
    }
    pub fn pad(&self) -> Pad {
        self.touch.pad()
    }
}
impl Default for PlayerView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Control, touch::Target};
    fn rect(left: f32, top: f32, right: f32, bottom: f32) -> Area {
        Area {
            left,
            top,
            right,
            bottom,
        }
    }
    fn window() -> WindowGeometry {
        WindowGeometry {
            window: rect(100.0, 200.0, 2380.0, 1280.0),
            surface: rect(100.0, 200.0, 2380.0, 1280.0),
            content: rect(100.0, 200.0, 2380.0, 1280.0),
            input_origin: (100.0, 200.0),
            clearance: Insets {
                left: 0.0,
                top: 66.0,
                right: 0.0,
                bottom: 66.0,
            },
            density: 2.75,
        }
    }
    #[test]
    fn nonzero_origins_inset_and_edge_to_edge_surfaces_have_the_same_screen_layout() {
        let w = window();
        let full = Geometry::new(w, 2280.0, 1080.0).unwrap();
        let mut inset = w;
        inset.surface = rect(100.0, 266.0, 2380.0, 1214.0);
        inset.content = inset.surface;
        let fitted = Geometry::new(inset, 2280.0, 948.0).unwrap();
        for (a, b) in full.layout.zones.iter().zip(&fitted.layout.zones) {
            assert_eq!(a.area.left, b.area.left);
            assert!((a.area.top - b.area.top - 66.0).abs() < 0.001);
        }
        assert_eq!(full.layout.game.top, 66.0);
        assert_eq!(fitted.layout.game.top, 0.0);
    }
    #[test]
    fn hidden_visible_bars_cutouts_density_and_scaled_buffer_keep_draw_and_touch_together() {
        for density in [1.0, 2.75, 3.0] {
            for cutout in [0.0, 80.0] {
                for right in [false, true] {
                    let mut w = window();
                    w.density = density;
                    if right {
                        w.clearance.right = cutout;
                    } else {
                        w.clearance.left = cutout;
                    }
                    // Hidden/transient bars keep measured stable clearance; visible
                    // bars can additionally inset content without changing safe bounds.
                    for visible in [false, true] {
                        if visible {
                            w.content = rect(
                                100.0 + w.clearance.left,
                                266.0,
                                2380.0 - w.clearance.right,
                                1214.0,
                            );
                        }
                        let mut view = PlayerView::new();
                        view.measure(Some(w));
                        let g = view.present(1140.0, 540.0).unwrap().clone();
                        for z in &g.layout.zones {
                            assert!(z.area.top >= 33.0 && z.area.bottom <= 507.01);
                            assert!(z.area.left >= cutout * 0.5 || right);
                            assert!(z.area.right <= 1140.0 - cutout * 0.5 || !right);
                            let (x, y) = z.area.center();
                            let input = (x * 2.0, y * 2.0);
                            let mapped = g.input_to_buffer(input.0, input.1);
                            assert!((mapped.0 - x).abs() < 0.001 && (mapped.1 - y).abs() < 0.001);
                            let pause = view.down(1, input.0, input.1);
                            assert_eq!(pause, z.target == Target::Pause);
                            if let Target::Button(c) = z.target {
                                assert_eq!(view.pad().values()[c as usize], 1.0);
                            }
                            view.cancel();
                        }
                        let picture = g.picture(4.0 / 3.0);
                        assert!(
                            ((picture.right - picture.left) / (picture.bottom - picture.top)
                                - 4.0 / 3.0)
                                .abs()
                                < 0.0001
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn geometry_change_or_invalid_transient_surface_neutralizes_held_input() {
        let mut view = PlayerView::new();
        let mut w = window();
        view.measure(Some(w));
        let a = view
            .present(2280.0, 1080.0)
            .unwrap()
            .layout
            .zones
            .iter()
            .find(|z| z.target == Target::Button(Control::A))
            .unwrap()
            .area
            .center();
        view.down(1, a.0, a.1);
        assert_eq!(view.pad().values()[0], 1.0);
        w.clearance.left = 80.0;
        view.measure(Some(w));
        assert!(view.pad().values().iter().all(|v| *v == 0.0));
        assert!(!view.down(2, a.0, a.1));
        assert!(view.pad().values().iter().all(|v| *v == 0.0));
        for (width, height) in [(0.0, 0.0), (1.0, 2.0), (f32::NAN, 1080.0)] {
            assert!(view.present(width, height).is_none());
        }
        assert!(view.present(2280.0, 1080.0).is_some());
        assert!(view.present(1080.0, 2280.0).is_none());
        view.measure(None);
        assert!(view.present(2280.0, 1080.0).is_none());
        w.content = rect(100.0, 200.0, 100.0, 200.0);
        assert!(Geometry::new(w, 2280.0, 1080.0).is_none());
    }
    #[test]
    fn inset_surface_input_origin_and_partial_travel_share_the_rendered_targets() {
        let mut w = window();
        w.surface = rect(130.0, 266.0, 2380.0, 1214.0);
        w.content = w.surface;
        let mut view = PlayerView::new();
        view.measure(Some(w));
        let g = view.present(1125.0, 474.0).unwrap().clone();
        let travel = g
            .layout
            .zones
            .iter()
            .find(|z| z.target == Target::Travel(Control::LTravel))
            .unwrap()
            .area;
        let click = g
            .layout
            .zones
            .iter()
            .find(|z| z.target == Target::Button(Control::LClick))
            .unwrap()
            .area;
        // Native motion is window-local, while this surface is offset by
        // (30,66) inside it; the buffer has half as many pixels in each axis.
        let input = |x: f32, y: f32| (x * 2.0 + 30.0, y * 2.0 + 66.0);
        let (x, y) = input(
            travel.left + (travel.right - travel.left) * 0.35,
            travel.center().1,
        );
        view.down(1, x, y);
        assert!((view.pad().values()[Control::LTravel as usize] - 0.35).abs() < 0.0001);
        assert_eq!(view.pad().values()[Control::LClick as usize], 0.0);
        let (x, y) = input(click.center().0, click.center().1);
        view.down(2, x, y);
        assert_eq!(view.pad().values()[Control::LClick as usize], 1.0);
        view.up(2);
        assert_eq!(view.pad().values()[Control::LClick as usize], 0.0);
        view.present(2250.0, 948.0);
        assert!(view.pad().values().iter().all(|v| *v == 0.0));
        // Short/transient but nonzero surfaces remain bounded, without NaNs.
        let short = Geometry::new(w, 200.0, 84.0).unwrap();
        for z in short.layout.zones {
            assert!(valid(z.area) && z.area.right <= 200.01 && z.area.bottom <= 84.01);
        }
    }
    #[test]
    fn title_free_content_reclaims_picture_height_without_spending_clearance_twice() {
        // A native window may cover framework chrome while its content view
        // still reserves that chrome. The app selects NoActionBar before window
        // creation; geometry must then use the newly measured full content.
        let mut w = window();
        w.content.top = 354.0;
        let chrome = Geometry::new(w, 2280.0, 1080.0).unwrap().picture(4.0 / 3.0);
        w.content = w.surface;
        let g = Geometry::new(w, 2280.0, 1080.0).unwrap();
        let picture = g.picture(4.0 / 3.0);
        assert_eq!(picture.top, 66.0);
        assert!((picture.bottom - 1014.0).abs() < 0.001);
        assert!(picture.bottom - picture.top > chrome.bottom - chrome.top);
        assert!((picture.right - picture.left) / (picture.bottom - picture.top) > 1.3332);
        for zone in g.layout.zones {
            assert!(zone.area.top >= 66.0 && zone.area.bottom <= 1014.01);
        }
    }
}
