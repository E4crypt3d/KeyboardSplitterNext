//! Mapping engine: turns "held physical keys of one player" into a single
//! controller report. Pure and platform independent (unit tested).

use std::collections::HashSet;

use crate::controller::{Accumulator, GamepadReport, accumulate, finish};
use crate::core::{
    Binding, DpadDirection, GamepadButton, StickDirection, StickSide, Target, TriggerSide,
};

fn button(button: GamepadButton) -> Target {
    Target::Button { button }
}
fn dpad(direction: DpadDirection) -> Target {
    Target::Dpad { direction }
}
fn trigger(side: TriggerSide) -> Target {
    Target::Trigger { side }
}
fn stick(side: StickSide, direction: StickDirection) -> Target {
    Target::Stick { side, direction }
}

/// Sensible starter layout for a player who owns their own keyboard.
/// Movement on the left stick (WASD) + aim on the right stick (arrows) is the
/// most common split-screen control scheme; the rest covers face buttons,
/// bumpers/triggers, D-pad and Start/Back. Every player slot starts with this
/// layout and the mapping editor can change individual keys.
pub fn default_bindings() -> Vec<Binding> {
    fn b(key: &str, target: Target) -> Binding {
        Binding { key: key.to_string(), target }
    }
    vec![
        b("W", stick(StickSide::Left, StickDirection::Up)),
        b("A", stick(StickSide::Left, StickDirection::Left)),
        b("S", stick(StickSide::Left, StickDirection::Down)),
        b("D", stick(StickSide::Left, StickDirection::Right)),
        b("Up", stick(StickSide::Right, StickDirection::Up)),
        b("Left", stick(StickSide::Right, StickDirection::Left)),
        b("Down", stick(StickSide::Right, StickDirection::Down)),
        b("Right", stick(StickSide::Right, StickDirection::Right)),
        b("Space", button(GamepadButton::A)),
        b("LShift", button(GamepadButton::B)),
        b("E", button(GamepadButton::X)),
        b("R", button(GamepadButton::Y)),
        b("Q", button(GamepadButton::LB)),
        b("F", button(GamepadButton::RB)),
        b("LCtrl", trigger(TriggerSide::Left)),
        b("Z", trigger(TriggerSide::Right)),
        b("Tab", button(GamepadButton::Back)),
        b("Enter", button(GamepadButton::Start)),
        b("I", dpad(DpadDirection::Up)),
        b("K", dpad(DpadDirection::Down)),
        b("J", dpad(DpadDirection::Left)),
        b("L", dpad(DpadDirection::Right)),
    ]
}

/// Compute the controller report for one player from its bindings and the
/// set of currently held canonical keys. Recomputed from scratch on every
/// event, so releasing W while S is held cannot leave a stuck axis.
pub fn report_for(bindings: &[Binding], held: &HashSet<String>) -> GamepadReport {
    let mut acc = Accumulator::default();
    for binding in bindings {
        if held.contains(&binding.key) {
            accumulate(binding.target, &mut acc);
        }
    }
    finish(acc)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held(keys: &[&str]) -> HashSet<String> {
        keys.iter().map(|k| k.to_string()).collect()
    }

    #[test]
    fn wasd_moves_left_stick() {
        let map = default_bindings();
        let r = report_for(&map, &held(&["W"]));
        assert_eq!(r.left_y, i16::MAX);
        let r = report_for(&map, &held(&["D"]));
        assert_eq!(r.left_x, i16::MAX);
    }

    #[test]
    fn space_presses_a() {
        let map = default_bindings();
        let r = report_for(&map, &held(&["Space"]));
        assert_eq!(r.buttons, crate::controller::BIT_A);
    }

    #[test]
    fn nothing_held_is_idle() {
        let map = default_bindings();
        let r = report_for(&map, &held(&[]));
        assert_eq!(r, GamepadReport::default());
    }

    #[test]
    fn w_and_s_cancel() {
        let map = default_bindings();
        let r = report_for(&map, &held(&["W", "S"]));
        assert_eq!(r.left_y, 0);
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let map = default_bindings();
        let r = report_for(&map, &held(&["F9", "W"]));
        assert_eq!(r.left_y, i16::MAX);
    }
}
