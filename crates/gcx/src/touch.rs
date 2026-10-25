// SPDX-License-Identifier: MIT
//! Position based GameCube touch policy; trigger travel never infers a click.
//! Landscape rail spacing adapts PSX's public MIT input geometry; see
//! docs/PSX_TRANSFER.md. GameCube targets and trigger policy remain independent.
use crate::{Control, Pad};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Target {
    Button(Control),
    Stick(Control, Control),
    Travel(Control),
    Dpad,
    Pause,
}
/// Result of the actual down-event path, without recording touch coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownReport {
    NoGeometry,
    Rejected,
    Miss,
    Captured,
    Pause,
}
impl Target {
    /// Original GameCube color cues, in the native draw list's ABGR format.
    pub fn color(self) -> u32 {
        match self {
            Self::Button(Control::A) => 0xff68b65a,
            Self::Button(Control::B) => 0xff4c53d9,
            Self::Button(Control::Z) => 0xffbc797b,
            Self::Button(_) | Self::Dpad => 0xffaaa6a2,
            _ => 0xb03c3b3a,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Zone {
    pub area: Area,
    pub target: Target,
    pub label: &'static CStrLabel,
}
pub type CStrLabel = std::ffi::CStr;
#[derive(Clone, Debug)]
pub struct Layout {
    pub game: Area,
    pub zones: Vec<Zone>,
}
impl Layout {
    pub fn new(width: f32, height: f32) -> Self {
        let width = width.max(1.0);
        let height = height.max(1.0);
        if width >= 600.0 && height >= 500.0 {
            return Self::docked(width, height);
        }
        if width > height * 1.25 {
            return Self::landscape(width, height);
        }
        if width >= 600.0 && height >= 280.0 {
            return Self::docked(width, height);
        }
        // A narrow display gets the same complete GameCube deck below the
        // picture. The old fractional portrait arrangement clipped bottom
        // targets and overlapped the C Stick. Keep real target sizes where a
        // normal phone can fit them; tiny transition surfaces scale as a unit.
        let deck_height = height.min(344.0);
        let mut layout = Self::arranged(width, deck_height, 352.0);
        let offset = height - deck_height;
        for zone in &mut layout.zones {
            zone.area.top += offset;
            zone.area.bottom += offset;
        }
        layout.game = Area {
            left: 0.0,
            top: 0.0,
            right: width,
            bottom: offset,
        };
        layout
    }
    /// Input and presentation use this same density-aware geometry. The engine
    /// still owns the final fit to its reported game aspect, never a PS1 ratio.
    pub fn for_display(width: f32, height: f32, density: f32) -> Self {
        Self::for_controls(width, height, density, None)
    }
    pub fn for_controls(
        width: f32,
        height: f32,
        density: f32,
        settings: Option<&crate::controls::Settings>,
    ) -> Self {
        let density = if density.is_finite() && density > 0.0 {
            density
        } else {
            1.0
        };
        let mut layout = settings.map_or_else(
            || Self::new(width / density, height / density),
            |s| s.layout(width / density, height / density),
        );
        let scale = |a: &mut Area| {
            a.left *= density;
            a.top *= density;
            a.right *= density;
            a.bottom *= density;
        };
        scale(&mut layout.game);
        for zone in &mut layout.zones {
            scale(&mut zone.area);
        }
        layout
    }
    fn landscape(width: f32, height: f32) -> Self {
        Self::arranged(width, height, 640.0)
    }
    fn docked(width: f32, height: f32) -> Self {
        // Keep the Fold's deck shallow enough for a full-width, aspect-correct
        // picture. The D-pad sits beside the Control Stick; C sits below-left
        // of A. Travel and digital shoulder clicks remain separate targets.
        let top = height - 172.0;
        let mut zones = Vec::new();
        let mut add = |x: f32, y: f32, w: f32, h: f32, target, label| {
            zones.push(Zone {
                area: Area {
                    left: x,
                    top: top + y,
                    right: x + w,
                    bottom: top + y + h,
                },
                target,
                label,
            })
        };
        for (x, travel, click, label, click_label) in [
            (8.0, Control::LTravel, Control::LClick, c"L analog", c"L"),
            (
                width - 384.0,
                Control::RTravel,
                Control::RClick,
                c"R analog",
                c"R",
            ),
        ] {
            add(x, 8.0, 104.0, 48.0, Target::Travel(travel), label);
            add(
                x + 112.0,
                8.0,
                48.0,
                48.0,
                Target::Button(click),
                click_label,
            );
        }
        add(
            8.0,
            60.0,
            104.0,
            104.0,
            Target::Stick(Control::StickX, Control::StickY),
            c"Control Stick",
        );
        add(136.0, 84.0, 80.0, 80.0, Target::Dpad, c"D-pad");
        add(
            width / 2.0 - 72.0,
            116.0,
            48.0,
            48.0,
            Target::Button(Control::Start),
            c"Start",
        );
        add(
            width / 2.0 - 16.0,
            116.0,
            48.0,
            48.0,
            Target::Pause,
            c"Pause",
        );
        for (x, y, size, control, label) in [
            (width - 64.0, 88.0, 64.0, Control::A, c"A"),
            (width - 104.0, 144.0, 48.0, Control::B, c"B"),
            (width - 32.0, 28.0, 48.0, Control::X, c"X"),
            (width - 128.0, 36.0, 48.0, Control::Y, c"Y"),
        ] {
            add(
                x - size / 2.0,
                y - size / 2.0,
                size,
                size,
                Target::Button(control),
                label,
            );
        }
        add(
            width - 208.0,
            8.0,
            48.0,
            48.0,
            Target::Button(Control::Z),
            c"Z",
        );
        add(
            width - 248.0,
            60.0,
            104.0,
            104.0,
            Target::Stick(Control::CX, Control::CY),
            c"C Stick",
        );
        Self {
            game: Area {
                left: 0.0,
                top: 0.0,
                right: width,
                bottom: top + 4.0,
            },
            zones,
        }
    }
    fn arranged(width: f32, height: f32, minimum_width: f32) -> Self {
        // PSX's side rails: density-independent thumb wells, a clear centered
        // picture, and separate reachable auxiliary keys. Scale only transient
        // small surfaces; ordinary phone targets remain at least 48 points.
        let scale = (width / minimum_width).min(height / 344.0).min(1.0);
        let w = width / scale;
        let h = height / scale;
        let rail = 176.0;
        let compact = h < 368.0;
        let gap = if compact { 4.0 } else { 8.0 };
        let upper = 56.0 + gap;
        let stick = 112.0;
        let lower = h - stick - 8.0;
        let mut zones = Vec::new();
        let mut add = |x, y, width, height, target, label| {
            zones.push(Zone {
                area: Area {
                    left: x * scale,
                    top: y * scale,
                    right: (x + width) * scale,
                    bottom: (y + height) * scale,
                },
                target,
                label,
            });
        };
        for (left, travel, click, travel_label, click_label) in [
            (true, Control::LTravel, Control::LClick, c"L analog", c"L"),
            (false, Control::RTravel, Control::RClick, c"R analog", c"R"),
        ] {
            let x = if left { 8.0 } else { w - rail + 8.0 };
            // Slider and click never share a target or an inferred threshold.
            add(x, 8.0, 104.0, 48.0, Target::Travel(travel), travel_label);
            add(
                x + 112.0,
                8.0,
                48.0,
                48.0,
                Target::Button(click),
                click_label,
            );
        }
        add(8.0, upper, 112.0, 112.0, Target::Dpad, c"D-pad");
        add(
            120.0,
            lower,
            48.0,
            48.0,
            Target::Button(Control::Start),
            c"Start",
        );
        add(120.0, lower + 52.0, 48.0, 48.0, Target::Pause, c"Pause");
        add(
            4.0,
            lower,
            stick,
            stick,
            Target::Stick(Control::StickX, Control::StickY),
            c"Control Stick",
        );
        let face = w - rail + 8.0;
        // A is the large primary cap, with B below-left and X/Y above-right/top.
        // Rectangular hit targets have real gaps, including around the large A.
        for (x, y, size, button, label) in [
            (84.0, 84.0, 64.0, Control::A, c"A"),
            (
                24.0,
                if compact { 92.0 } else { 104.0 },
                48.0,
                Control::B,
                c"B",
            ),
            (140.0, 58.0, 48.0, Control::X, c"X"),
            (82.0, 24.0, 48.0, Control::Y, c"Y"),
        ] {
            add(
                face + x - size / 2.0,
                upper + y - size / 2.0,
                size,
                size,
                Target::Button(button),
                label,
            );
        }
        add(
            w - rail + 8.0,
            lower,
            48.0,
            48.0,
            Target::Button(Control::Z),
            c"Z",
        );
        add(
            w - 4.0 - stick,
            lower,
            stick,
            stick,
            Target::Stick(Control::CX, Control::CY),
            c"C Stick",
        );
        Self {
            game: Area {
                left: rail * scale,
                top: 0.0,
                right: (w - rail) * scale,
                bottom: height,
            },
            zones,
        }
    }
}
#[derive(Clone, Copy)]
struct Capture {
    zone: Zone,
    x: f32,
    y: f32,
    origin: (f32, f32),
    direction: Option<u8>,
    dead_zone: f32,
    following: bool,
}
impl Capture {
    fn track(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        if matches!(self.zone.target, Target::Stick(..) | Target::Dpad) {
            // Adapt PSX's following thumb center, without its stick-click gesture.
            let reach = ((self.zone.area.right - self.zone.area.left)
                .min(self.zone.area.bottom - self.zone.area.top)
                * 0.32)
                .max(1.0);
            let (dx, dy) = (x - self.origin.0, y - self.origin.1);
            let length = dx.hypot(dy);
            if length > reach && (self.following || self.zone.target == Target::Dpad) {
                self.origin = (x - dx * reach / length, y - dy * reach / length);
            }
            if self.zone.target == Target::Dpad {
                let (dx, dy) = (x - self.origin.0, y - self.origin.1);
                let dead = (self.zone.area.right - self.zone.area.left) * 0.09;
                let threshold = if self.direction.is_some() {
                    dead * 0.75
                } else {
                    dead
                };
                if dx.hypot(dy) <= threshold {
                    self.direction = None;
                } else {
                    let angle = dy.atan2(dx);
                    let sector = std::f32::consts::FRAC_PI_4;
                    let keep = self.direction.is_some_and(|index| {
                        let delta = (angle - f32::from(index) * sector + std::f32::consts::PI)
                            .rem_euclid(std::f32::consts::TAU)
                            - std::f32::consts::PI;
                        delta.abs() <= sector * 0.5 + 5_f32.to_radians()
                    });
                    if !keep {
                        self.direction = Some((angle / sector).round().rem_euclid(8.0) as u8);
                    }
                }
            }
        }
    }
    fn axes(self) -> (f32, f32) {
        let reach = ((self.zone.area.right - self.zone.area.left)
            .min(self.zone.area.bottom - self.zone.area.top)
            * 0.32)
            .max(1.0);
        let (mut x, mut y) = (
            (self.x - self.origin.0) / reach,
            (self.origin.1 - self.y) / reach,
        );
        let length = x.hypot(y);
        if length <= self.dead_zone {
            return (0.0, 0.0);
        }
        // Continuous radial dead zone, followed by the GameCube octagonal gate.
        let scale = ((length - self.dead_zone) / (1.0 - self.dead_zone)).min(1.0) / length;
        x *= scale;
        y *= scale;
        let boundary =
            x.abs().max(y.abs()) + (std::f32::consts::SQRT_2 - 1.0) * x.abs().min(y.abs());
        if boundary > 1.0 {
            x /= boundary;
            y /= boundary;
        }
        (x, y)
    }
    fn directions(self) -> [bool; 4] {
        // Up, Down, Left, Right. No console-specific bit mask is shared.
        match self.direction {
            Some(0) => [false, false, false, true],
            Some(1) => [false, true, false, true],
            Some(2) => [false, true, false, false],
            Some(3) => [false, true, true, false],
            Some(4) => [false, false, true, false],
            Some(5) => [true, false, true, false],
            Some(6) => [true, false, false, false],
            Some(7) => [true, false, false, true],
            _ => [false; 4],
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct DeliveryStats {
    pub native_polls: u64,
    pub max_poll_gap_micros: u64,
    pub source_events: u64,
    pub invalid_source_times: u64,
    pub max_event_to_dispatch_micros: u64,
    pub event_histogram: [u64; 7],
    pub updates: u64,
    pub sampled_batches: u64,
    pub canceled_batches: u64,
    pub max_dispatch_to_poll_micros: u64,
    /// <=1,2,4,8,16,33ms, then >33ms. Software dispatch to native pad poll only.
    pub histogram: [u64; 7],
}
#[derive(Default)]
pub struct TouchPad {
    captures: [Option<(i32, Capture)>; 16],
    pending: [bool; 12],
    oldest_update: Option<Instant>,
    last_poll: Option<Instant>,
    delivery: DeliveryStats,
    dead_zone: f32,
    following: bool,
}
impl TouchPad {
    pub const fn new() -> Self {
        Self {
            captures: [None; 16],
            pending: [false; 12],
            oldest_update: None,
            last_poll: None,
            dead_zone: 0.12,
            following: true,
            delivery: DeliveryStats {
                native_polls: 0,
                max_poll_gap_micros: 0,
                source_events: 0,
                invalid_source_times: 0,
                max_event_to_dispatch_micros: 0,
                event_histogram: [0; 7],
                updates: 0,
                sampled_batches: 0,
                canceled_batches: 0,
                max_dispatch_to_poll_micros: 0,
                histogram: [0; 7],
            },
        }
    }
    pub fn configure(&mut self, dead_zone_percent: u8, following: bool) {
        self.cancel();
        self.dead_zone = f32::from(dead_zone_percent.min(25)) / 100.0;
        self.following = following;
    }
    /// A captured pointer keeps its target while it moves. Layout changes cancel captures.
    pub fn down(&mut self, id: i32, x: f32, y: f32, layout: &Layout) -> bool {
        self.down_report(id, x, y, layout) == DownReport::Pause
    }
    pub fn down_report(&mut self, id: i32, x: f32, y: f32, layout: &Layout) -> DownReport {
        if !x.is_finite()
            || !y.is_finite()
            || self.captures.iter().all(Option::is_some)
            || self
                .captures
                .iter()
                .flatten()
                .any(|(pointer, _)| *pointer == id)
        {
            return DownReport::Rejected;
        }
        if let Some(zone) = layout
            .zones
            .iter()
            .rev()
            .find(|z| z.area.contains(x, y))
            .copied()
        {
            if matches!(zone.target, Target::Pause) {
                return DownReport::Pause;
            }
            if matches!(
                zone.target,
                Target::Stick(..) | Target::Travel(_) | Target::Dpad
            ) && self
                .captures
                .iter()
                .flatten()
                .any(|(_, capture)| capture.zone.target == zone.target)
            {
                return DownReport::Rejected;
            }
            let mut capture = Capture {
                zone,
                x,
                y,
                origin: zone.area.center(),
                direction: None,
                dead_zone: self.dead_zone,
                following: self.following,
            };
            capture.track(x, y);
            self.latch(capture);
            *self
                .captures
                .iter_mut()
                .find(|slot| slot.is_none())
                .unwrap() = Some((id, capture));
            self.updated();
            return DownReport::Captured;
        }
        DownReport::Miss
    }
    pub fn moved(&mut self, id: i32, x: f32, y: f32) {
        if x.is_finite()
            && y.is_finite()
            && let Some((_, c)) = self
                .captures
                .iter_mut()
                .flatten()
                .find(|(pointer, _)| *pointer == id)
        {
            c.track(x, y);
            let capture = *c;
            self.latch(capture);
            self.updated();
        }
    }
    pub fn up(&mut self, id: i32) {
        if let Some(slot) = self
            .captures
            .iter_mut()
            .find(|slot| slot.is_some_and(|(pointer, _)| pointer == id))
        {
            *slot = None;
            self.updated();
        }
    }
    pub fn cancel(&mut self) {
        self.captures.fill(None);
        self.pending.fill(false);
        self.last_poll = None; // Deliberate pause/departure is not a polling stall.
        if self.oldest_update.take().is_some() {
            self.delivery.canceled_batches += 1;
        }
    }
    fn updated(&mut self) {
        self.oldest_update.get_or_insert_with(Instant::now);
        self.delivery.updates += 1;
    }
    fn latch(&mut self, capture: Capture) {
        if let Target::Button(control) = capture.zone.target {
            if let Some(pending) = self.pending.get_mut(control as usize) {
                *pending = true;
            }
        } else if capture.zone.target == Target::Dpad && capture.direction.is_some() {
            self.pending[Control::Up as usize..=Control::Right as usize]
                .copy_from_slice(&capture.directions());
        }
    }
    pub fn delivery(&self) -> DeliveryStats {
        self.delivery
    }
    pub fn begin_runtime(&mut self) {
        self.cancel();
        self.delivery = DeliveryStats::default();
    }
    /// Native Android event timestamp to Rust handling, independently of pad polls.
    pub fn record_source_age(&mut self, age: Option<Duration>) {
        let Some(age) = age else {
            self.delivery.invalid_source_times += 1;
            return;
        };
        self.delivery.source_events += 1;
        self.delivery.event_histogram[Self::bucket(age)] += 1;
        self.delivery.max_event_to_dispatch_micros = self
            .delivery
            .max_event_to_dispatch_micros
            .max(age.as_micros().min(u128::from(u64::MAX)) as u64);
    }
    fn bucket(wait: Duration) -> usize {
        [1, 2, 4, 8, 16, 33]
            .iter()
            .position(|&ms| wait <= Duration::from_millis(ms))
            .unwrap_or(6)
    }
    pub fn stick_center(&self, axis: Control) -> Option<(f32, f32)> {
        self.captures.iter().flatten().find_map(|(_, c)| {
            matches!(c.zone.target, Target::Stick(x, _) if x == axis).then_some(c.origin)
        })
    }
    /// Only the actual native pad poll consumes pulses; drawing and host ticks do not.
    pub fn sample_pad(&mut self) -> Pad {
        self.sample_at(Instant::now())
    }
    fn sample_at(&mut self, now: Instant) -> Pad {
        self.delivery.native_polls += 1;
        if let Some(previous) = self.last_poll.replace(now) {
            self.delivery.max_poll_gap_micros = self.delivery.max_poll_gap_micros.max(
                now.saturating_duration_since(previous)
                    .as_micros()
                    .min(u128::from(u64::MAX)) as u64,
            );
        }
        let mut pad = self.pad();
        for (value, pending) in pad.0.iter_mut().zip(&mut self.pending) {
            if std::mem::take(pending) {
                *value = 1.0;
            }
        }
        if let Some(at) = self.oldest_update.take() {
            let wait = now.saturating_duration_since(at);
            let bucket = Self::bucket(wait);
            self.delivery.histogram[bucket] += 1;
            self.delivery.sampled_batches += 1;
            self.delivery.max_dispatch_to_poll_micros = self
                .delivery
                .max_dispatch_to_poll_micros
                .max(wait.as_micros().min(u128::from(u64::MAX)) as u64);
        }
        pad
    }
    pub fn pad(&self) -> Pad {
        let mut pad = Pad::default();
        for (_, c) in self.captures.iter().flatten() {
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
                    let (x, y) = c.axes();
                    let _ = pad.set(cx, f64::from(x));
                    let _ = pad.set(cy, f64::from(y));
                }
                Target::Dpad => {
                    for (control, pressed) in
                        [Control::Up, Control::Down, Control::Left, Control::Right]
                            .into_iter()
                            .zip(c.directions())
                    {
                        if pressed {
                            let _ = pad.set(control, 1.0);
                        }
                    }
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
    fn fold_and_narrow_decks_keep_all_targets_inside_with_independent_gamecube_controls() {
        for (w, h) in [
            (800., 700.),
            (840., 700.),
            (700., 800.),
            (390., 844.),
            (352., 700.),
        ] {
            let layout = Layout::new(w, h);
            for (i, z) in layout.zones.iter().enumerate() {
                let a = z.area;
                assert!(
                    a.left >= 0. && a.top >= 0. && a.right <= w + 0.01 && a.bottom <= h + 0.01,
                    "{w}x{h} {:?}",
                    z.target
                );
                assert!(
                    a.right - a.left >= 47.99 && a.bottom - a.top >= 47.99,
                    "{w}x{h} {:?}",
                    z.target
                );
                for other in &layout.zones[..i] {
                    let b = other.area;
                    assert!(
                        !(a.left < b.right - 0.01
                            && a.right > b.left + 0.01
                            && a.top < b.bottom - 0.01
                            && a.bottom > b.top + 0.01),
                        "{w}x{h} {:?} and {:?}",
                        z.target,
                        other.target
                    );
                }
            }
            for target in [
                Target::Stick(Control::StickX, Control::StickY),
                Target::Stick(Control::CX, Control::CY),
                Target::Dpad,
                Target::Travel(Control::LTravel),
                Target::Travel(Control::RTravel),
                Target::Button(Control::LClick),
                Target::Button(Control::RClick),
                Target::Button(Control::Z),
                Target::Button(Control::Start),
                Target::Pause,
            ] {
                assert!(layout.zones.iter().any(|z| z.target == target));
            }
        }
    }
    fn zone(layout: &Layout, target: Target) -> Area {
        layout
            .zones
            .iter()
            .find(|z| z.target == target)
            .unwrap()
            .area
    }
    #[test]
    fn primary_sticks_share_the_bottom_thumb_band_with_clear_auxiliary_keys() {
        let layout = Layout::for_display(2280.0, 948.0, 2.75);
        let left = zone(&layout, Target::Stick(Control::StickX, Control::StickY));
        let right = zone(&layout, Target::Stick(Control::CX, Control::CY));
        let dpad = zone(&layout, Target::Dpad);
        let start = zone(&layout, Target::Button(Control::Start));
        let pause = zone(&layout, Target::Pause);
        assert_eq!(left.top, right.top);
        assert!(left.bottom > 948.0 * 0.9 && dpad.bottom < left.top);
        assert!(start.left > left.right && pause.left > left.right);
    }
    #[test]
    fn rapid_press_survives_host_ticks_and_render_reads_until_native_poll_once() {
        let layout = Layout::new(830.0, 393.0);
        let mut touch = TouchPad::new();
        for (id, control) in [(1, Control::A), (2, Control::B), (3, Control::RClick)] {
            let (x, y) = zone(&layout, Target::Button(control)).center();
            touch.down(id, x, y, &layout);
            touch.up(id);
        }
        for _ in 0..20 {
            assert_eq!(touch.pad().values(), &[0.0; 18]);
        }
        let polled = touch.sample_pad();
        for c in [Control::A, Control::B, Control::RClick] {
            assert_eq!(polled.values()[c as usize], 1.0);
        }
        assert_eq!(polled.values()[Control::RTravel as usize], 0.0);
        assert_eq!(touch.sample_pad().values(), &[0.0; 18]);
        let (x, y) = zone(&layout, Target::Button(Control::A)).center();
        touch.down(4, x, y, &layout);
        assert_eq!(touch.sample_pad().values()[Control::A as usize], 1.0);
        assert_eq!(touch.sample_pad().values()[Control::A as usize], 1.0);
        touch.up(4);
        assert_eq!(touch.sample_pad().values()[Control::A as usize], 0.0);
        touch.down(5, x, y, &layout);
        touch.up(5);
        touch.cancel();
        assert_eq!(touch.sample_pad().values(), &[0.0; 18]);
    }
    #[test]
    fn sticks_have_continuous_dead_zone_gamecube_gate_and_following_reverse() {
        let layout = Layout::new(830.0, 393.0);
        for (cx, cy) in [
            (Control::StickX, Control::StickY),
            (Control::CX, Control::CY),
        ] {
            let a = zone(&layout, Target::Stick(cx, cy));
            let (x, y) = a.center();
            let reach = (a.right - a.left) * 0.32;
            let mut touch = TouchPad::new();
            touch.down(1, x, y, &layout);
            let mut previous = 0.0;
            for i in 0..=100 {
                touch.moved(1, x + reach * i as f32 / 100.0, y);
                let axis = touch.pad().values()[cx as usize];
                assert!(axis >= previous && axis - previous < 0.012);
                previous = axis;
            }
            assert!((previous - 1.0).abs() < 0.0001);
            touch.moved(1, x + reach * 4.0, y);
            touch.moved(1, x + reach * 3.8, y);
            assert!(touch.pad().values()[cx as usize] < 0.8);
            touch.up(1);
            touch.down(2, x, y, &layout);
            touch.moved(2, x + reach, y - reach);
            let pad = touch.pad();
            assert!((pad.values()[cx as usize] - std::f64::consts::FRAC_1_SQRT_2).abs() < 0.001);
            assert!((pad.values()[cy as usize] - std::f64::consts::FRAC_1_SQRT_2).abs() < 0.001);
            touch.cancel();
            assert_eq!(touch.sample_pad().values(), &[0.0; 18]);
        }
    }
    #[test]
    fn dpad_has_angular_hysteresis_and_does_not_steal_a_second_pointer() {
        let layout = Layout::new(830.0, 393.0);
        let a = zone(&layout, Target::Dpad);
        let (x, y) = a.center();
        let reach = (a.right - a.left) * 0.25;
        let mut touch = TouchPad::new();
        touch.down(1, x + reach, y, &layout);
        for degrees in [22.0_f32, 24.0, 26.0, 23.0] {
            let angle = degrees.to_radians();
            touch.moved(1, x + reach * angle.cos(), y + reach * angle.sin());
            assert_eq!(touch.pad().values()[Control::Down as usize], 0.0);
        }
        let angle = 30_f32.to_radians();
        touch.moved(1, x + reach * angle.cos(), y + reach * angle.sin());
        assert_eq!(touch.pad().values()[Control::Down as usize], 1.0);
        assert_eq!(touch.pad().values()[Control::Right as usize], 1.0);
        touch.down(2, x - reach, y, &layout);
        assert_eq!(touch.pad().values()[Control::Left as usize], 0.0);
        touch.cancel();
        assert_eq!(touch.sample_pad().values(), &[0.0; 18]);
    }
    #[test]
    fn dispatch_age_statistics_are_bounded_and_do_not_count_game_response() {
        let mut touch = TouchPad::new();
        let at = Instant::now();
        for delay in [1, 2, 3, 7, 15, 30, 34] {
            touch.record_source_age(Some(Duration::from_millis(delay)));
            touch.oldest_update = Some(at);
            touch.sample_at(at + Duration::from_millis(delay));
        }
        let s = touch.delivery();
        assert_eq!(s.sampled_batches, 7);
        assert_eq!(s.histogram, [1; 7]);
        assert_eq!(s.max_dispatch_to_poll_micros, 34000);
        assert_eq!(s.source_events, 7);
        assert_eq!(s.event_histogram, [1; 7]);
        assert_eq!(s.max_event_to_dispatch_micros, 34000);
        touch.record_source_age(None);
        assert_eq!(touch.delivery().invalid_source_times, 1);
        touch.oldest_update = Some(at);
        touch.cancel();
        touch.sample_at(at + Duration::from_secs(1));
        assert_eq!(touch.delivery().canceled_batches, 1);
        assert_eq!(touch.delivery().sampled_batches, 7);
        assert_eq!(touch.delivery().max_poll_gap_micros, 15000);
        touch.begin_runtime();
        assert_eq!(touch.delivery().native_polls, 0);
    }
    #[test]
    fn landscape_rails_keep_every_target_clear_and_density_independent() {
        for (width, height, density) in [
            (2280.0, 1080.0, 2.75),
            // A short phone drawable after ordinary system-bar clearance.
            (2280.0, 948.0, 2.75),
            (2342.0, 1021.0, 2.625),
            (1920.0, 1080.0, 3.0),
        ] {
            let layout = Layout::for_display(width, height, density);
            for (i, zone) in layout.zones.iter().enumerate() {
                let a = zone.area;
                assert!(a.left >= 0.0 && a.top >= 0.0 && a.right <= width && a.bottom <= height);
                assert!(a.right <= layout.game.left || a.left >= layout.game.right);
                assert!((a.right - a.left).min(a.bottom - a.top) / density >= 47.9);
                for other in &layout.zones[i + 1..] {
                    let b = other.area;
                    assert!(
                        a.right <= b.left
                            || b.right <= a.left
                            || a.bottom <= b.top
                            || b.bottom <= a.top,
                        "overlapping targets: {:?} / {:?}",
                        zone.target,
                        other.target
                    );
                }
            }
            // All 18 engine channels are independently reachable; no PS1 mask
            // or invented stick-click inputs participate in this layout.
            let mut observed = [false; 18];
            for (id, zone) in layout.zones.iter().enumerate() {
                for (x, y) in [
                    zone.area.center(),
                    (zone.area.right, zone.area.top),
                    (zone.area.left, zone.area.bottom),
                ] {
                    let mut touch = TouchPad::new();
                    touch.down(
                        id as i32,
                        zone.area.center().0,
                        zone.area.center().1,
                        &layout,
                    );
                    touch.moved(id as i32, x, y);
                    for (i, value) in touch.pad().values().iter().enumerate() {
                        observed[i] |= value.abs() > 0.0;
                    }
                }
            }
            assert!(observed.iter().all(|v| *v));
        }
    }
    #[test]
    fn landscape_dpad_diagonal_and_independent_trigger_release() {
        let layout = Layout::new(830.0, 393.0);
        let dpad = layout
            .zones
            .iter()
            .find(|z| z.target == Target::Dpad)
            .unwrap()
            .area;
        let travel = layout
            .zones
            .iter()
            .find(|z| z.target == Target::Travel(Control::RTravel))
            .unwrap()
            .area;
        let click = layout
            .zones
            .iter()
            .find(|z| z.target == Target::Button(Control::RClick))
            .unwrap()
            .area;
        let mut touch = TouchPad::new();
        touch.down(1, dpad.right - 1.0, dpad.top + 1.0, &layout);
        touch.down(
            2,
            travel.left + (travel.right - travel.left) * 0.35,
            travel.center().1,
            &layout,
        );
        assert_eq!(touch.pad().values()[Control::Up as usize], 1.0);
        assert_eq!(touch.pad().values()[Control::Right as usize], 1.0);
        assert_eq!(touch.pad().values()[Control::RClick as usize], 0.0);
        touch.down(3, click.center().0, click.center().1, &layout);
        touch.up(3);
        assert!((touch.pad().values()[Control::RTravel as usize] - 0.35).abs() < 1e-5);
        touch.cancel();
        assert!(touch.pad().values().iter().all(|v| *v == 0.0));
    }
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
