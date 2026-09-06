//! Mapping engine: turns "held physical keys of one player" into a single
//! controller report. Pure and platform independent (unit tested).
//!
//! Starter layouts ("presets") for new players and for popular games live in
//! `crate::presets`; `default_bindings` is simply the "general" preset.

use std::collections::HashSet;

use crate::controller::{Accumulator, GamepadReport, accumulate, finish};
use crate::core::Binding;

pub use crate::presets::default_bindings;

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

    #[test]
    fn two_players_with_the_same_preset_stay_independent() {
        // The "each keyboard is its own controller" flow: every player keeps
        // its own held-key set and its own report, so two players can run the
        // very same preset on two keyboards. Pressing the same keys on both
        // must drive two identical-but-separate reports, and a press on one
        // player's keyboard must never leak into the other player's report.
        let map = crate::presets::bindings("football-pes").unwrap();

        // Both players hold the same keys on their own keyboards: move + pass.
        let p1 = report_for(&map, &held(&["W", "Space"]));
        let p2 = report_for(&map, &held(&["W", "Space"]));
        assert_eq!(p1, p2, "identical inputs on two keyboards = identical reports");
        assert_eq!(p1.buttons, crate::controller::BIT_A);

        // Player 2 now also taps shoot (E). Only p2's report changes.
        let p2_shot = report_for(&map, &held(&["W", "Space", "E"]));
        let p1_unchanged = report_for(&map, &held(&["W", "Space"]));
        assert_eq!(p1_unchanged, p1, "player 1's pad must not react to player 2's keys");
        // On the eFootball/PES (Konami Standard) preset, shoot = Xbox X.
        assert_eq!(p2_shot.buttons, crate::controller::BIT_A | crate::controller::BIT_X);
        assert_ne!(p2_shot, p1_unchanged);
    }
}
