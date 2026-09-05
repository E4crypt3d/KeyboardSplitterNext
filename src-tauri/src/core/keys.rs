//! Canonical physical-key naming.
//!
//! Raw Input reports Windows virtual-key codes (`VKey`). Because a mapping
//! profile should stay readable and stable across machines, every key is
//! stored under a canonical name such as `"W"`, `"Space"`, `"Left"`,
//! `"Num4"`, `"F5"`. The table below is layout agnostic - like every other
//! keyboard-based game tool it refers to physical US-layout positions
//! (the same codes games poll).

/// Virtual key code of the *first* function key (F1).
const VK_F1: u16 = 0x70;
/// Virtual key code of the first main-row digit key.
const VK_DIGIT0: u16 = 0x30;
/// Virtual key code of the first letter key.
const VK_A: u16 = 0x41;

/// Keys that have a dedicated display name. Lookup is linear; the table is tiny.
const NAMED_KEYS: &[(u16, &str)] = &[
    (0x08, "Backspace"),
    (0x09, "Tab"),
    (0x0D, "Enter"),
    (0x13, "Pause"),
    (0x14, "CapsLock"),
    (0x1B, "Escape"),
    (0x20, "Space"),
    (0x21, "PageUp"),
    (0x22, "PageDown"),
    (0x23, "End"),
    (0x24, "Home"),
    (0x25, "Left"),
    (0x26, "Up"),
    (0x27, "Right"),
    (0x28, "Down"),
    (0x2C, "PrintScreen"),
    (0x2D, "Insert"),
    (0x2E, "Delete"),
    (0x5B, "LWin"),
    (0x5C, "RWin"),
    (0x5D, "Apps"),
    (0x60, "Num0"),
    (0x61, "Num1"),
    (0x62, "Num2"),
    (0x63, "Num3"),
    (0x64, "Num4"),
    (0x65, "Num5"),
    (0x66, "Num6"),
    (0x67, "Num7"),
    (0x68, "Num8"),
    (0x69, "Num9"),
    (0x6A, "NumMultiply"),
    (0x6B, "NumAdd"),
    (0x6C, "NumSeparator"),
    (0x6D, "NumSubtract"),
    (0x6E, "NumDecimal"),
    (0x6F, "NumDivide"),
    (0x90, "NumLock"),
    (0xA0, "LShift"),
    (0xA1, "RShift"),
    (0xA2, "LCtrl"),
    (0xA3, "RCtrl"),
    (0xA4, "LAlt"),
    (0xA5, "RAlt"),
    (0xBA, ";"),
    (0xBB, "="),
    (0xBC, ","),
    (0xBD, "-"),
    (0xBE, "."),
    (0xBF, "/"),
    (0xC0, "`"),
    (0xDB, "["),
    (0xDC, "\\"),
    (0xDD, "]"),
    (0xDE, "'"),
];

/// Convert a Windows virtual key code into a canonical name.
pub fn name_for_vk(vk: u16) -> String {
    if (VK_DIGIT0..=VK_DIGIT0 + 9).contains(&vk) {
        return ((b'0' + (vk - VK_DIGIT0) as u8) as char).to_string();
    }
    if (VK_A..=VK_A + 25).contains(&vk) {
        return ((b'A' + (vk - VK_A) as u8) as char).to_string();
    }
    if (VK_F1..=VK_F1 + 23).contains(&vk) {
        return format!("F{}", vk - VK_F1 + 1);
    }
    for (code, name) in NAMED_KEYS {
        if *code == vk {
            return name.to_string();
        }
    }
    format!("VK_{vk:02X}")
}

/// Convert a canonical name back into a Windows virtual key code.
/// (Reserved for future numeric import/export - raw events only produce
/// names through `name_for_vk`, but keeping the reverse lookup tested.)
#[allow(dead_code)]
pub fn vk_for_name(name: &str) -> Option<u16> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let bytes = name.as_bytes();
    if bytes.len() == 1 {
        let b = bytes[0];
        if b.is_ascii_uppercase() {
            return Some(VK_A + (b - b'A') as u16);
        }
        if b.is_ascii_digit() {
            return Some(VK_DIGIT0 + (b - b'0') as u16);
        }
    }
    if name.len() > 1 && name.starts_with('F') {
        if let Ok(n) = name[1..].parse::<u16>() {
            if (1..=24).contains(&n) {
                return Some(VK_F1 + n - 1);
            }
        }
    }
    if name.starts_with("Num") && name.len() == 4 {
        if let Some(d) = name.as_bytes()[3].checked_sub(b'0') {
            if d <= 9 {
                return Some(0x60 + d as u16);
            }
        }
    }
    for (code, n) in NAMED_KEYS {
        if *n == name {
            return Some(*code);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_and_digits_round_trip() {
        assert_eq!(name_for_vk(0x41), "A");
        assert_eq!(name_for_vk(0x5A), "Z");
        assert_eq!(name_for_vk(0x30), "0");
        assert_eq!(vk_for_name("W"), Some(0x57));
        assert_eq!(vk_for_name("7"), Some(0x37));
    }

    #[test]
    fn named_keys_round_trip() {
        assert_eq!(name_for_vk(0x20), "Space");
        assert_eq!(vk_for_name("Space"), Some(0x20));
        assert_eq!(name_for_vk(0x25), "Left");
        assert_eq!(vk_for_name("LCtrl"), Some(0xA2));
    }

    #[test]
    fn function_keys() {
        assert_eq!(name_for_vk(0x70), "F1");
        assert_eq!(name_for_vk(0x87), "F24");
        assert_eq!(vk_for_name("F12"), Some(0x7B));
    }

    #[test]
    fn unknown_keys_get_stable_fallback() {
        assert!(name_for_vk(0xEE).starts_with("VK_"));
    }
}
