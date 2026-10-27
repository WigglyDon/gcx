// SPDX-License-Identifier: MIT
//! The stationary stock test gesture must reach the real touch-policy poll.
use gcx::{
    Control,
    touch::{DownReport, Layout, Target, TouchPad},
};

#[test]
fn displaced_down_reaches_the_first_poll_and_stays_held_until_release() {
    let layout = Layout::new(830.0, 393.0);
    for floating in [false, true] {
        for (axis_x, axis_y) in [
            (Control::StickX, Control::StickY),
            (Control::CX, Control::CY),
        ] {
            let area = layout
                .zones
                .iter()
                .find(|zone| zone.target == Target::Stick(axis_x, axis_y))
                .unwrap()
                .area;
            let (center_x, center_y) = area.center();
            let reach = (area.right - area.left).min(area.bottom - area.top) * 0.32;
            let (x, y) = (center_x + reach * 0.65, center_y - reach * 0.35);
            let mut touch = TouchPad::new();
            touch.configure(12, floating);
            assert_eq!(touch.down_report(1, x, y, &layout), DownReport::Captured);
            let first = *touch.sample_pad().values();
            assert!(first[axis_x as usize] > 0.3);
            assert!(first[axis_y as usize] > 0.1);
            for _ in 0..16 {
                touch.moved(1, x, y);
                assert_eq!(touch.sample_pad().values(), &first);
            }
            touch.up(1);
            assert_eq!(touch.sample_pad().values(), &[0.0; 18]);
        }
    }
}

#[test]
fn interruption_neutralizes_both_stationary_sticks_and_rejects_old_moves() {
    let layout = Layout::new(830.0, 393.0);
    let mut touch = TouchPad::new();
    let mut contacts = Vec::new();
    for (pointer, axis_x, axis_y) in [
        (1, Control::StickX, Control::StickY),
        (2, Control::CX, Control::CY),
    ] {
        let area = layout
            .zones
            .iter()
            .find(|zone| zone.target == Target::Stick(axis_x, axis_y))
            .unwrap()
            .area;
        let (center_x, center_y) = area.center();
        let x = center_x + (area.right - area.left) * 0.2;
        assert_eq!(
            touch.down_report(pointer, x, center_y, &layout),
            DownReport::Captured
        );
        contacts.push((pointer, x, center_y));
    }
    let held = touch.sample_pad();
    assert!(held.values()[Control::StickX as usize] > 0.3);
    assert!(held.values()[Control::CX as usize] > 0.3);
    touch.cancel();
    assert_eq!(touch.sample_pad().values(), &[0.0; 18]);
    for (pointer, x, y) in contacts {
        touch.moved(pointer, x, y);
        touch.up(pointer);
    }
    assert_eq!(touch.sample_pad().values(), &[0.0; 18]);
}

#[test]
fn digital_l_click_never_infers_analog_trigger_travel() {
    let layout = Layout::new(830.0, 393.0);
    let area = layout
        .zones
        .iter()
        .find(|zone| zone.target == Target::Button(Control::LClick))
        .unwrap()
        .area;
    let (x, y) = area.center();
    let mut touch = TouchPad::new();
    assert_eq!(touch.down_report(1, x, y, &layout), DownReport::Captured);
    let pad = touch.sample_pad();
    assert_eq!(pad.values()[Control::LClick as usize], 1.0);
    for control in [Control::LTravel, Control::RTravel, Control::RClick] {
        assert_eq!(pad.values()[control as usize], 0.0);
    }
    touch.up(1);
    assert_eq!(touch.sample_pad().values(), &[0.0; 18]);
}
