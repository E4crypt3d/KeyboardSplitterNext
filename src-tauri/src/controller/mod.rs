//! Virtual controller abstraction.
//!
//! The engine only knows about [`VirtualController`] and [`GamepadReport`].
//! The concrete backend (ViGEmBus, see `vigem.rs`) can be swapped without
//! touching engine / mapping / UI code (engineering rule #2 + #3).

use crate::core::{DpadDirection, GamepadButton, StickDirection, StickSide, Target, TriggerSide};

// XInput button bit layout (identical to the Xbox 360 XUSB report and to the
// bits ViGEmBus expects).
pub const BIT_DPAD_UP: u16 = 0x0001;
pub const BIT_DPAD_DOWN: u16 = 0x0002;
pub const BIT_DPAD_LEFT: u16 = 0x0004;
pub const BIT_DPAD_RIGHT: u16 = 0x0008;
pub const BIT_START: u16 = 0x0010;
pub const BIT_BACK: u16 = 0x0020;
pub const BIT_LTHUMB: u16 = 0x0040;
pub const BIT_RTHUMB: u16 = 0x0080;
pub const BIT_LB: u16 = 0x0100;
pub const BIT_RB: u16 = 0x0200;
pub const BIT_GUIDE: u16 = 0x0400;
pub const BIT_A: u16 = 0x1000;
pub const BIT_B: u16 = 0x2000;
pub const BIT_X: u16 = 0x4000;
pub const BIT_Y: u16 = 0x8000;

/// Full XInput-compatible controller state that gets pushed to the driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamepadReport {
    pub buttons: u16,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub left_x: i16,
    pub left_y: i16,
    pub right_x: i16,
    pub right_y: i16,
}

/// Intermediate accumulation buffer used while combining every pressed
/// binding of one player into a single report.
#[derive(Debug, Clone, Copy, Default)]
pub struct Accumulator {
    pub buttons: u16,
    pub left_trigger: u32,
    pub right_trigger: u32,
    pub left_x: i32,
    pub left_y: i32,
    pub right_x: i32,
    pub right_y: i32,
}

pub fn button_bit(button: GamepadButton) -> u16 {
    match button {
        GamepadButton::A => BIT_A,
        GamepadButton::B => BIT_B,
        GamepadButton::X => BIT_X,
        GamepadButton::Y => BIT_Y,
        GamepadButton::LB => BIT_LB,
        GamepadButton::RB => BIT_RB,
        GamepadButton::Back => BIT_BACK,
        GamepadButton::Start => BIT_START,
        GamepadButton::LThumb => BIT_LTHUMB,
        GamepadButton::RThumb => BIT_RTHUMB,
        GamepadButton::Guide => BIT_GUIDE,
    }
}

pub fn dpad_bit(dir: DpadDirection) -> u16 {
    match dir {
        DpadDirection::Up => BIT_DPAD_UP,
        DpadDirection::Down => BIT_DPAD_DOWN,
        DpadDirection::Left => BIT_DPAD_LEFT,
        DpadDirection::Right => BIT_DPAD_RIGHT,
    }
}

/// Add the contribution of one target to the accumulator.
/// Sticks/triggers are summed (W+S cancel out), buttons are OR-ed.
pub fn accumulate(target: Target, acc: &mut Accumulator) {
    match target {
        Target::Button { button } => acc.buttons |= button_bit(button),
        Target::Dpad { direction } => acc.buttons |= dpad_bit(direction),
        Target::Trigger { side } => match side {
            TriggerSide::Left => acc.left_trigger = acc.left_trigger.saturating_add(255),
            TriggerSide::Right => acc.right_trigger = acc.right_trigger.saturating_add(255),
        },
        Target::Stick { side, direction } => {
            let (dx, dy): (i32, i32) = match direction {
                StickDirection::Up => (0, 1),
                StickDirection::Down => (0, -1),
                StickDirection::Left => (-1, 0),
                StickDirection::Right => (1, 0),
            };
            // XInput reports sticks as full-range i16: -32768 .. 32767.
            // A digital key should push all the way in its direction. Values
            // are summed first (W+S cancel out) and clamped in `finish`.
            let mag = 32768i32;
            match side {
                StickSide::Left => {
                    acc.left_x += dx * mag;
                    acc.left_y += dy * mag;
                }
                StickSide::Right => {
                    acc.right_x += dx * mag;
                    acc.right_y += dy * mag;
                }
            }
        }
    }
}

/// Finalize an accumulator into a driver-ready report.
pub fn finish(acc: Accumulator) -> GamepadReport {
    GamepadReport {
        buttons: acc.buttons,
        left_trigger: acc.left_trigger.min(u8::MAX as u32) as u8,
        right_trigger: acc.right_trigger.min(u8::MAX as u32) as u8,
        left_x: acc.left_x.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        left_y: acc.left_y.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        right_x: acc.right_x.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        right_y: acc.right_y.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
    }
}

/// A connected virtual Xbox controller.
///
/// Implementations are responsible for driver plumbing only. `submit` pushes
/// a full report; engines are expected to only submit when something changed.
/// ViGEmBus driver backend (Windows only).
#[cfg(target_os = "windows")]
pub mod vigem;

pub trait VirtualController: Send {
    /// Connect / (re)create the virtual controller. Returns a user readable
    /// error on failure (e.g. ViGEmBus driver not installed).
    fn connect(&mut self) -> Result<(), String>;
    /// Unplug / destroy the virtual controller.
    fn disconnect(&mut self);
    /// Push a full report to the virtual controller.
    fn submit(&mut self, report: &GamepadReport) -> Result<(), String>;
    fn is_connected(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{GamepadButton, StickDirection, StickSide, Target, TriggerSide};

    fn one(target: Target) -> GamepadReport {
        let mut acc = Accumulator::default();
        accumulate(target, &mut acc);
        finish(acc)
    }

    #[test]
    fn digital_button_sets_bit() {
        let r = one(Target::Button { button: GamepadButton::A });
        assert_eq!(r.buttons, BIT_A);
    }

    #[test]
    fn stick_direction_pushes_full_deflection() {
        let r = one(Target::Stick { side: StickSide::Left, direction: StickDirection::Up });
        assert_eq!(r.left_y, i16::MAX);
        assert_eq!(r.left_x, 0);

        let r = one(Target::Stick { side: StickSide::Right, direction: StickDirection::Left });
        assert_eq!(r.right_x, i16::MIN);
    }

    #[test]
    fn trigger_is_full_analog_press() {
        let r = one(Target::Trigger { side: TriggerSide::Right });
        assert_eq!(r.right_trigger, 255);
    }

    #[test]
    fn opposite_sticks_cancel_out() {
        let mut acc = Accumulator::default();
        accumulate(
            Target::Stick { side: StickSide::Left, direction: StickDirection::Up },
            &mut acc,
        );
        accumulate(
            Target::Stick { side: StickSide::Left, direction: StickDirection::Down },
            &mut acc,
        );
        let r = finish(acc);
        assert_eq!(r.left_y, 0);
    }
}
