//! Physical keyboard detection through the Windows Raw Input API.
//!
//! Raw Input is the standard, driver-free way to enumerate keyboards and to
//! tell which physical device produced a key (unlike a regular key hook,
//! raw input carries the source device handle). Detection is split from
//! capture (see `capture.rs`).

use crate::core::KeyboardDevice;

/// Extract vendor/product id from a Windows device path such as
/// `\\?\USB#VID_046D&PID_C52B#6&2a3b4c&0&1#{...}`.
///
/// Panic-free by construction: some drivers expose paths where the string
/// ends right after (or inside) the `VID_`/`PID_` marker, so the hex slice
/// is only taken through `str::get`, which returns `None` instead of
/// panicking on out-of-range or non-character-boundary indices. This ran on
/// the raw-input thread at startup and on every device plug/unplug, and a
/// panic here used to abort the whole process with a memory/heap error.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn parse_vid_pid(path: &str) -> (u32, u32) {
    let hex_at = |marker: &str| -> u32 {
        path.find(marker)
            .and_then(|i| path.get(i + marker.len()..i + marker.len() + 4))
            .and_then(|hex| u32::from_str_radix(hex, 16).ok())
            .unwrap_or(0)
    };
    (hex_at("VID_"), hex_at("PID_"))
}

#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use std::collections::HashSet;
    use std::mem::{size_of, zeroed};

    use windows::Win32::Foundation::{GetLastError, HANDLE};
    use windows::Win32::UI::Input::{
        GetRawInputDeviceInfoW, GetRawInputDeviceList, RAWINPUTDEVICELIST, RID_DEVICE_INFO,
        RID_DEVICE_INFO_HID, RIDI_DEVICEINFO, RIDI_DEVICENAME, RIM_TYPEHID, RIM_TYPEKEYBOARD,
    };

    const USAGE_PAGE_GENERIC_DESKTOP: u16 = 0x01;
    const USAGE_KEYBOARD: u16 = 0x06;

    fn device_path(handle: HANDLE) -> Option<String> {
        unsafe {
            let mut size: u32 = 0;
            GetRawInputDeviceInfoW(Some(handle), RIDI_DEVICENAME, None, &mut size);
            if size == 0 || size > 1 << 16 {
                return None;
            }
            // First call reports the required size in bytes; ask again with a
            // slightly larger buffer to be safe.
            let mut buf = vec![0u16; (size as usize / 2) + 4];
            let mut buf_size = (buf.len() * 2) as u32;
            let ret = GetRawInputDeviceInfoW(
                Some(handle),
                RIDI_DEVICENAME,
                Some(buf.as_mut_ptr() as *mut _),
                &mut buf_size,
            );
            if ret == 0 {
                return None;
            }
            let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            Some(String::from_utf16_lossy(&buf[..len]))
        }
    }

    fn device_info(handle: HANDLE) -> Option<RID_DEVICE_INFO> {
        unsafe {
            let mut info: RID_DEVICE_INFO = RID_DEVICE_INFO {
                cbSize: size_of::<RID_DEVICE_INFO>() as u32,
                dwType: zeroed(),
                Anonymous: zeroed(),
            };
            let mut size = size_of::<RID_DEVICE_INFO>() as u32;
            let ret = GetRawInputDeviceInfoW(
                Some(handle),
                RIDI_DEVICEINFO,
                Some(&mut info as *mut RID_DEVICE_INFO as *mut _),
                &mut size,
            );
            if ret == 0 {
                return None;
            }
            Some(info)
        }
    }

    fn friendly_name(path: &str, vid: u32, pid: u32) -> String {
        if vid != 0 {
            return format!("Keyboard ({vid:04X}:{pid:04X})");
        }
        // Fall back to the last meaningful path segment: ACPI#PNP0303 -> PNP0303
        let segment = path
            .split('#')
            .find(|s| !s.is_empty() && !s.starts_with('{'))
            .and_then(|s| s.rsplit('\\').next())
            .unwrap_or("Keyboard");
        if segment.to_ascii_lowercase().contains("keyboard") {
            segment.to_string()
        } else {
            format!("Keyboard ({segment})")
        }
    }

    fn is_keyboard_type(info: &RID_DEVICE_INFO, dw_type: u32) -> bool {
        match dw_type {
            t if t == RIM_TYPEKEYBOARD.0 => true,
            t if t == RIM_TYPEHID.0 => unsafe {
                let hid: RID_DEVICE_INFO_HID = info.Anonymous.hid;
                hid.usUsagePage == USAGE_PAGE_GENERIC_DESKTOP && hid.usUsage == USAGE_KEYBOARD
            },
            _ => false,
        }
    }

    /// Enumerate keyboards, returning each device together with its raw input
    /// device handle (the handle lets the capture thread map WM_INPUT events
    /// back to a specific keyboard).
    #[allow(dead_code)]
    pub fn enumerate_with_handles() -> Vec<(usize, KeyboardDevice)> {
        unsafe {
            let mut count: u32 = 0;
            let entry_size = size_of::<RAWINPUTDEVICELIST>() as u32;
            GetRawInputDeviceList(None, &mut count, entry_size);
            if count == 0 {
                return Vec::new();
            }
            let mut list: Vec<RAWINPUTDEVICELIST> = vec![zeroed(); count as usize];
            let written = GetRawInputDeviceList(Some(list.as_mut_ptr()), &mut count, entry_size);
            if written == u32::MAX {
                log::error!("GetRawInputDeviceList failed: {}", GetLastError().0);
                return Vec::new();
            }
            list.truncate(written as usize);

            let mut seen = HashSet::new();
            let mut devices: Vec<(usize, KeyboardDevice)> = Vec::new();

            for entry in list {
                let Some(path) = device_path(entry.hDevice) else { continue };
                let Some(info) = device_info(entry.hDevice) else { continue };

                let dw_type = info.dwType.0;
                if !is_keyboard_type(&info, dw_type) {
                    continue;
                }
                // HID keyboards often register twice (TYPEKEYBOARD + TYPEHID
                // with the same path); deduplicate by path.
                if !seen.insert(path.clone()) {
                    continue;
                }

                let (mut vid, mut pid) = parse_vid_pid(&path);
                if vid == 0 && pid == 0 {
                    if dw_type == RIM_TYPEHID.0 {
                        let hid: RID_DEVICE_INFO_HID = info.Anonymous.hid;
                        vid = hid.dwVendorId;
                        pid = hid.dwProductId;
                    }
                }

                devices.push((
                    entry.hDevice.0 as usize,
                    KeyboardDevice {
                        id: path.clone(),
                        name: friendly_name(&path, vid, pid),
                        vendor_id: vid,
                        product_id: pid,
                        path,
                    },
                ));
            }

            devices.sort_by(|a, b| a.1.name.cmp(&b.1.name));
            devices
        }
    }

    #[allow(dead_code)]
    pub fn enumerate() -> Vec<KeyboardDevice> {
        enumerate_with_handles().into_iter().map(|(_, d)| d).collect()
    }
}

#[cfg(not(target_os = "windows"))]
// Linux/macOS stubs: nothing enumerates them (the capture thread is Windows-only),
// so silence dead-code there while keeping the Windows build strict.
#[allow(dead_code)]
mod imp {
    use super::*;

    pub fn enumerate() -> Vec<KeyboardDevice> {
        Vec::new()
    }
    pub fn enumerate_with_handles() -> Vec<(usize, KeyboardDevice)> {
        Vec::new()
    }
}

#[allow(unused_imports)]
pub use imp::enumerate;
#[allow(unused_imports)]
pub(crate) use imp::enumerate_with_handles;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vid_pid_from_typical_usb_paths() {
        let path = r"\\?\USB#VID_046D&PID_C52B#6&2a3b4c&0&1#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}";
        assert_eq!(parse_vid_pid(path), (0x046D, 0xC52B));
    }

    #[test]
    fn truncated_markers_do_not_panic() {
        // Paths that end right after or inside the marker: slicing used to
        // panic here and take the whole process down (memory/heap error).
        // Fewer than 4 hex digits available means no parseable id -> 0.
        assert_eq!(parse_vid_pid(r"\\?\ACPI#VID_"), (0, 0));
        assert_eq!(parse_vid_pid(r"\\?\ACPI#PID_1"), (0, 0));
        assert_eq!(parse_vid_pid(r"\\?\ACPI#VID_04"), (0, 0));
        assert_eq!(parse_vid_pid(r"\\?\ACPI#PID_123"), (0, 0));
        // A full 4-digit marker at the very end of the path still parses.
        assert_eq!(parse_vid_pid(r"\\?\ACPI#VID_046D"), (0x046D, 0));
        assert_eq!(parse_vid_pid(r"\\?\ACPI#PID_C52B"), (0, 0xC52B));
    }

    #[test]
    fn non_hex_and_missing_markers_yield_zero() {
        assert_eq!(parse_vid_pid(r"\\?\ACPI#PNP0303#..."), (0, 0));
        assert_eq!(parse_vid_pid(""), (0, 0));
        assert_eq!(parse_vid_pid(r"\\?\USB#VID_ZZZZ&PID_0000#x"), (0, 0));
    }

    #[test]
    fn takes_the_first_marker_occurrence() {
        assert_eq!(parse_vid_pid("VID_0001&PID_0002&VID_0003"), (0x0001, 0x0002));
    }
}
