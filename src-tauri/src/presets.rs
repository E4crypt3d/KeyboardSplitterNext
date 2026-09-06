//! Built-in starter layouts ("presets") for popular local-multiplayer games.
//!
//! A preset is a per-player key → controller mapping template. Because each
//! player owns a separate physical keyboard, the *same* preset can be applied
//! to every player and both still press their own keys (unlike classic
//! one-keyboard splitters, where players had to use different zones).
//!
//! Controller-side layouts follow the games' documented Xbox defaults:
//!   - EA Sports FC / FIFA:      A pass · B shoot · X cross/lob · Y through (in-game "Classic")
//!   - eFootball / PES:          A low pass · X shoot · B lofted/cross · Y through, dash on RB
//!                               (Konami "Standard" - also the out-of-the-box PES 2017 layout;
//!                               eFootball's "Alternate" scheme is the EA-style one above)
//!   - Mortal Kombat 1/11:       X/Y/A/B = FP/BP/FK/BK · LB throw · RT block
//!   - Tekken 7/8:               X/Y/A/B = LP/RP/LK/RK (combos are key chords)
//!   - Street Fighter 6:         X/Y/RB = LP/MP/HP · A/B/RT = LK/MK/HK
//!
//! The keyboard keys themselves are free (per-player keyboards), so presets
//! only encode an ergonomic, non-conflicting arrangement. Game defaults can
//! be tweaked in-game; presets are starting points, not gospel.

use serde::{Deserialize, Serialize};

use crate::core::{
    Binding, DpadDirection, GamepadButton, StickDirection, StickSide, Target, TriggerSide,
};

/// Public id of the default layout (also used for new player slots).
pub const GENERAL_ID: &str = "general";

// ---------------------------------------------------------------------------
// Preset metadata + binding tables
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetMeta {
    pub id: String,
    pub name: String,
    pub description: String,
    pub key_count: usize,
}

struct PresetDef {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    /// (canonical key name, target). Key order is display order in the UI.
    keys: &'static [(&'static str, Target)],
}

const fn b(button: GamepadButton) -> Target {
    Target::Button { button }
}
const fn dpad(dir: DpadDirection) -> Target {
    Target::Dpad { direction: dir }
}
const fn trig(side: TriggerSide) -> Target {
    Target::Trigger { side }
}
const fn stick(side: StickSide, dir: StickDirection) -> Target {
    Target::Stick { side, direction: dir }
}

/// Movement on WASD (left hand), extra/aim controls on the arrows.
const GENERAL_KEYS: &[(&str, Target)] = &[
    ("W", stick(StickSide::Left, StickDirection::Up)),
    ("A", stick(StickSide::Left, StickDirection::Left)),
    ("S", stick(StickSide::Left, StickDirection::Down)),
    ("D", stick(StickSide::Left, StickDirection::Right)),
    ("Up", stick(StickSide::Right, StickDirection::Up)),
    ("Left", stick(StickSide::Right, StickDirection::Left)),
    ("Down", stick(StickSide::Right, StickDirection::Down)),
    ("Right", stick(StickSide::Right, StickDirection::Right)),
    ("Space", b(GamepadButton::A)),
    ("LShift", b(GamepadButton::B)),
    ("E", b(GamepadButton::X)),
    ("R", b(GamepadButton::Y)),
    ("Q", b(GamepadButton::LB)),
    ("F", b(GamepadButton::RB)),
    ("LCtrl", trig(TriggerSide::Left)),
    ("Z", trig(TriggerSide::Right)),
    ("Tab", b(GamepadButton::Back)),
    ("Enter", b(GamepadButton::Start)),
    ("I", dpad(DpadDirection::Up)),
    ("K", dpad(DpadDirection::Down)),
    ("J", dpad(DpadDirection::Left)),
    ("L", dpad(DpadDirection::Right)),
];

/// Football: EA Sports FC / FIFA, in-game "Classic" scheme (the default).
/// Movement on the left stick, passing/shooting cluster on the left hand:
/// A pass · B shoot · X cross/lob · Y through ball. Dash/sprint is the right
/// trigger (C). EA's in-game "Alternate" scheme (sprint on RB, shoot on X)
/// matches the eFootball/PES Standard preset instead.
const FOOTBALL_FC_KEYS: &[(&str, Target)] = &[
    ("W", stick(StickSide::Left, StickDirection::Up)),
    ("A", stick(StickSide::Left, StickDirection::Left)),
    ("S", stick(StickSide::Left, StickDirection::Down)),
    ("D", stick(StickSide::Left, StickDirection::Right)),
    ("Space", b(GamepadButton::A)), // pass / header
    ("E", b(GamepadButton::B)),     // shoot
    ("Q", b(GamepadButton::X)),     // cross / lob pass
    ("R", b(GamepadButton::Y)),     // through ball
    ("Tab", b(GamepadButton::LB)),  // change player
    ("F", b(GamepadButton::RB)),    // teammate press / call for support (hold)
    ("LCtrl", trig(TriggerSide::Left)), // shield / jockey (hold)
    ("C", trig(TriggerSide::Right)),    // sprint / dash (hold)
    ("Up", dpad(DpadDirection::Up)),
    ("Left", dpad(DpadDirection::Left)),
    ("Down", dpad(DpadDirection::Down)),
    ("Right", dpad(DpadDirection::Right)),
    ("Enter", b(GamepadButton::Start)), // pause
];

/// Football: eFootball / PES with the Konami "Standard" scheme that PES 2017,
/// PES 2021 and eFootball ship out of the box (verified against the games'
/// control charts/manuals): A low pass · X shoot · B lofted pass/cross ·
/// Y through ball, dash on RB, specials on RT. Note the Xbox labels are
/// positional - the PlayStation ▢/○/△ faces sit on Xbox X/B/Y. eFootball's
/// in-game "Alternate" scheme is the EA-style one (shoot B, dash RT); use
/// the EA FC preset for that feel.
const FOOTBALL_PES_KEYS: &[(&str, Target)] = &[
    ("W", stick(StickSide::Left, StickDirection::Up)),
    ("A", stick(StickSide::Left, StickDirection::Left)),
    ("S", stick(StickSide::Left, StickDirection::Down)),
    ("D", stick(StickSide::Left, StickDirection::Right)),
    ("Space", b(GamepadButton::A)), // low/short pass (pressure when defending)
    ("E", b(GamepadButton::X)),     // shoot (Xbox X = PlayStation ▢ position)
    ("Q", b(GamepadButton::B)),     // lofted pass / cross
    ("R", b(GamepadButton::Y)),     // through ball
    ("Tab", b(GamepadButton::LB)),  // cursor change / change player
    ("F", b(GamepadButton::RB)),    // dash / sprint (hold)
    ("C", trig(TriggerSide::Right)),    // special controls / skill modifier (hold)
    ("LCtrl", trig(TriggerSide::Left)), // manual controls (hold)
    ("Up", dpad(DpadDirection::Up)),
    ("Left", dpad(DpadDirection::Left)),
    ("Down", dpad(DpadDirection::Down)),
    ("Right", dpad(DpadDirection::Right)),
    ("Enter", b(GamepadButton::Start)), // pause
];

/// Fighting: Mortal Kombat 1 / 11. Xbox default: X=front punch, Y=back punch,
/// A=front kick, B=back kick, LB=throw, RT=block, LT=stance (hold).
const MK_KEYS: &[(&str, Target)] = &[
    ("W", stick(StickSide::Left, StickDirection::Up)),
    ("A", stick(StickSide::Left, StickDirection::Left)),
    ("S", stick(StickSide::Left, StickDirection::Down)),
    ("D", stick(StickSide::Left, StickDirection::Right)),
    ("U", b(GamepadButton::X)), // front punch
    ("I", b(GamepadButton::Y)), // back punch
    ("J", b(GamepadButton::A)), // front kick
    ("K", b(GamepadButton::B)), // back kick
    ("O", b(GamepadButton::LB)), // throw
    ("L", b(GamepadButton::RB)), // kameo assist (MK1) / interact
    ("LCtrl", trig(TriggerSide::Left)), // flawless block / stance
    (";", trig(TriggerSide::Right)),    // block (hold)
    ("Enter", b(GamepadButton::Start)), // pause
    ("Tab", b(GamepadButton::Back)),
];

/// Fighting: Tekken 7 / 8. Xbox default: X=LP(1), Y=RP(2), A=LK(3), B=RK(4).
/// Grabs / specials are button chords (e.g. 1+3) - press two keys together.
const TEKKEN_KEYS: &[(&str, Target)] = &[
    ("W", stick(StickSide::Left, StickDirection::Up)),
    ("A", stick(StickSide::Left, StickDirection::Left)),
    ("S", stick(StickSide::Left, StickDirection::Down)),
    ("D", stick(StickSide::Left, StickDirection::Right)),
    ("U", b(GamepadButton::X)), // 1 = left punch
    ("I", b(GamepadButton::Y)), // 2 = right punch
    ("J", b(GamepadButton::A)), // 3 = left kick
    ("K", b(GamepadButton::B)), // 4 = right kick
    ("Enter", b(GamepadButton::Start)), // pause / skip
    ("Tab", b(GamepadButton::Back)),
];

/// Fighting: Street Fighter 6. Xbox default: X=LP, Y=MP, RB=HP,
/// A=LK, B=MK, RT=HK. Drive moves are chords (MP+MK parry, HP+HK DI).
// NOTE: SF6 binds heavy kick to the right trigger; the engine's trigger
// target is "full press while held", which matches a single tap just fine.
const SF6_KEYS: &[(&str, Target)] = &[
    ("W", stick(StickSide::Left, StickDirection::Up)),
    ("A", stick(StickSide::Left, StickDirection::Left)),
    ("S", stick(StickSide::Left, StickDirection::Down)),
    ("D", stick(StickSide::Left, StickDirection::Right)),
    ("U", b(GamepadButton::X)),     // light punch
    ("I", b(GamepadButton::Y)),     // medium punch
    ("O", b(GamepadButton::RB)),    // heavy punch
    ("J", b(GamepadButton::A)),     // light kick
    ("K", b(GamepadButton::B)),     // medium kick
    ("L", trig(TriggerSide::Right)), // heavy kick (hold)
    ("Enter", b(GamepadButton::Start)), // pause
    ("Tab", b(GamepadButton::Back)),
];

/// Co-op platformers (Cuphead, Overcooked, It Takes Two, ...). Classic
/// "player 1 = left side of the keyboard" ergonomics, per-player repeatable.
const PLATFORM_COOP_KEYS: &[(&str, Target)] = &[
    ("W", stick(StickSide::Left, StickDirection::Up)),
    ("A", stick(StickSide::Left, StickDirection::Left)),
    ("S", stick(StickSide::Left, StickDirection::Down)),
    ("D", stick(StickSide::Left, StickDirection::Right)),
    ("Space", b(GamepadButton::A)), // jump
    ("E", b(GamepadButton::X)),     // interact / pick up
    ("R", b(GamepadButton::Y)),     // dash / ability
    ("F", b(GamepadButton::RB)),    // shoot / throw
    ("Q", b(GamepadButton::LB)),    // aim / drop
    ("LShift", b(GamepadButton::B)), // run / crouch
    ("LCtrl", trig(TriggerSide::Left)),
    ("C", trig(TriggerSide::Right)),
    ("Enter", b(GamepadButton::Start)), // pause
    ("Tab", b(GamepadButton::Back)),
];

/// Arcade racing (Crash Team Racing, Hot Wheels Unleashed, Trackmania, ...).
/// Steering on the left stick, full-press triggers for brake / gas.
const RACING_KEYS: &[(&str, Target)] = &[
    ("W", stick(StickSide::Left, StickDirection::Up)),
    ("A", stick(StickSide::Left, StickDirection::Left)),
    ("S", stick(StickSide::Left, StickDirection::Down)),
    ("D", stick(StickSide::Left, StickDirection::Right)),
    ("LCtrl", trig(TriggerSide::Left)), // brake
    ("C", trig(TriggerSide::Right)),    // accelerate (hold)
    ("Space", b(GamepadButton::A)),     // handbrake / drift
    ("E", b(GamepadButton::X)),         // item / camera
    ("F", b(GamepadButton::RB)),        // boost / item
    ("Q", b(GamepadButton::LB)),        // look back
    ("LShift", b(GamepadButton::B)),    // brake alt / reverse
    ("Enter", b(GamepadButton::Start)), // pause / menu
    ("Tab", b(GamepadButton::Back)),
];

/// Twin-stick arena shooters (Enter the Gungeon, Nuclear Throne, ...):
/// WASD move on LS, arrows aim on RS, fire on a trigger.
const TWIN_STICK_KEYS: &[(&str, Target)] = &[
    ("W", stick(StickSide::Left, StickDirection::Up)),
    ("A", stick(StickSide::Left, StickDirection::Left)),
    ("S", stick(StickSide::Left, StickDirection::Down)),
    ("D", stick(StickSide::Left, StickDirection::Right)),
    ("Up", stick(StickSide::Right, StickDirection::Up)),
    ("Left", stick(StickSide::Right, StickDirection::Left)),
    ("Down", stick(StickSide::Right, StickDirection::Down)),
    ("Right", stick(StickSide::Right, StickDirection::Right)),
    ("Space", b(GamepadButton::A)), // dodge / roll
    ("C", trig(TriggerSide::Right)), // fire (hold)
    ("LCtrl", trig(TriggerSide::Left)), // alt fire
    ("Q", b(GamepadButton::LB)),    // swap weapon
    ("F", b(GamepadButton::RB)),    // active item
    ("E", b(GamepadButton::X)),     // interact / open
    ("Enter", b(GamepadButton::Start)), // pause
    ("Tab", b(GamepadButton::Back)),
];

/// Third-person / platform shooters with aim: WASD move, arrows look,
/// trigger shoot, shoulder aim - classic controller muscle memory.
const FPS_GAMEPAD_KEYS: &[(&str, Target)] = &[
    ("W", stick(StickSide::Left, StickDirection::Up)),
    ("A", stick(StickSide::Left, StickDirection::Left)),
    ("S", stick(StickSide::Left, StickDirection::Down)),
    ("D", stick(StickSide::Left, StickDirection::Right)),
    ("Up", stick(StickSide::Right, StickDirection::Up)),
    ("Left", stick(StickSide::Right, StickDirection::Left)),
    ("Down", stick(StickSide::Right, StickDirection::Down)),
    ("Right", stick(StickSide::Right, StickDirection::Right)),
    ("Space", b(GamepadButton::A)),     // jump
    ("LShift", b(GamepadButton::LB)),   // aim down sights (hold)
    ("C", trig(TriggerSide::Right)),    // fire (hold)
    ("LCtrl", trig(TriggerSide::Left)), // crouch / slide
    ("R", b(GamepadButton::Y)),         // reload / ability
    ("E", b(GamepadButton::X)),         // interact / swap
    ("F", b(GamepadButton::RB)),        // melee / grenade
    ("Enter", b(GamepadButton::Start)), // pause
    ("Tab", b(GamepadButton::Back)),
];

static PRESETS: &[PresetDef] = &[
    PresetDef {
        id: GENERAL_ID,
        name: "General (WASD + arrows)",
        description: "All-round starter: movement on WASD, right stick on the arrows, face buttons around Space/E/R and triggers on Ctrl/Z. Great default for most games.",
        keys: GENERAL_KEYS,
    },
    PresetDef {
        id: "football-fc",
        name: "Football (FIFA / EA FC)",
        description: "EA FC defaults (in-game Classic scheme): pass Space, shoot E, cross Q, through R, sprint C (right trigger), teammate press F, change player Tab. Shooting feels wrong in PES? That game's default puts shoot on X - use the eFootball/PES preset instead.",
        keys: FOOTBALL_FC_KEYS,
    },
    PresetDef {
        id: "football-pes",
        name: "Football (eFootball / PES)",
        description: "PES/eFootball out-of-the-box layout (Konami Standard, same scheme as PES 2017): pass Space, shoot E, cross Q, through R, dash F (RB), specials C (RT), change player Tab. If eFootball is set to its Alternate scheme the buttons are EA-style - apply the EA FC preset instead.",
        keys: FOOTBALL_PES_KEYS,
    },
    PresetDef {
        id: "mortal-kombat",
        name: "Fighting (Mortal Kombat)",
        description: "MK 1/11 pad layout: punches/kicks on U I J K, throw on O, block on ; (right trigger). Reach every button without lifting a finger.",
        keys: MK_KEYS,
    },
    PresetDef {
        id: "tekken",
        name: "Fighting (Tekken)",
        description: "Tekken 7/8: LP/RP/LK/RK on U I J K (X/Y/A/B). Grabs and 1+3/2+4 moves are chords - press two keys at once, like two face buttons.",
        keys: TEKKEN_KEYS,
    },
    PresetDef {
        id: "street-fighter",
        name: "Fighting (Street Fighter 6)",
        description: "SF6 six-button layout: LP/MP/HP on U I O, LK/MK on J K and heavy kick on the right trigger. Drive parry/impact are chords.",
        keys: SF6_KEYS,
    },
    PresetDef {
        id: "platform-coop",
        name: "Co-op platformer",
        description: "Cuphead, Overcooked, It Takes Two and friends: jump on Space, interact on E, dash/shoot on R/F, run on Shift.",
        keys: PLATFORM_COOP_KEYS,
    },
    PresetDef {
        id: "racing",
        name: "Arcade racing",
        description: "Steer with WASD, hold C to accelerate and Ctrl to brake, Space for handbrake/drift - old-school arcade controls that never ghost.",
        keys: RACING_KEYS,
    },
    PresetDef {
        id: "twin-stick",
        name: "Twin-stick shooter",
        description: "Move on WASD, aim on the arrows, fire on C. For Enter the Gungeon, Nuclear Throne and other dodge-and-shoot co-op games.",
        keys: TWIN_STICK_KEYS,
    },
    PresetDef {
        id: "fps-gamepad",
        name: "Shooter (gamepad style)",
        description: "Move on WASD, look on the arrows, hold Shift to aim and C to fire - a controller feel for co-op shooters on one PC.",
        keys: FPS_GAMEPAD_KEYS,
    },
];

/// All built-in presets with metadata (used by `list_presets`).
pub fn list() -> Vec<PresetMeta> {
    PRESETS
        .iter()
        .map(|p| PresetMeta {
            id: p.id.to_string(),
            name: p.name.to_string(),
            description: p.description.to_string(),
            key_count: p.keys.len(),
        })
        .collect()
}

/// The bindings of a built-in preset.
pub fn bindings(id: &str) -> Option<Vec<Binding>> {
    PRESETS
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.keys.iter().map(|(k, t)| Binding { key: k.to_string(), target: *t }).collect())
}


/// Default layout for new players == the "general" preset.
pub fn default_bindings() -> Vec<Binding> {
    bindings(GENERAL_ID).expect("general preset must exist")
}

// ---------------------------------------------------------------------------
// Edge-case validation
// ---------------------------------------------------------------------------

/// True if `key` could conflict with Windows/OS shortcuts when pressed while
/// the game has focus (presets must stay away from these). Also the keys that
/// "press a key to assign" treats as cancel, so they must never be bindable.
#[cfg(test)]
fn is_os_reserved(key: &str) -> bool {
    matches!(key, "LWin" | "RWin" | "Apps" | "LAlt" | "RAlt" | "Escape")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    // Canonical-key round-trip helpers (test-only: production code only ever
    // produces names through `name_for_vk` from raw input events).
    use crate::core::keys::{name_for_vk, vk_for_name};

    #[test]
    fn general_is_the_default() {
        assert_eq!(GENERAL_ID, "general");
        assert_eq!(super::default_bindings().len(), GENERAL_KEYS.len());
        // stays the historical starter layout
        let names: Vec<String> =
            super::default_bindings().iter().map(|b| b.key.clone()).collect();
        assert!(names.contains(&"W".to_string()));
        assert!(names.contains(&"Space".to_string()));
    }

    #[test]
    fn bindings_mirror_the_preset_definitions() {
        for def in PRESETS {
            let expected: Vec<Binding> = def
                .keys
                .iter()
                .map(|(k, t)| Binding { key: k.to_string(), target: *t })
                .collect();
            assert_eq!(
                super::bindings(def.id).as_ref(),
                Some(&expected),
                "preset {}",
                def.id
            );
        }
        assert_eq!(super::bindings("nope"), None);
    }

    #[test]
    fn every_preset_has_a_distinct_key_target_table() {
        // The preset picker identifies the preset a player is using by
        // comparing (key → target) pairs, so two presets that bind the same
        // keys to the same targets would be indistinguishable (this is what
        // made the PES preset "snap back" to FIFA: they shared the same key
        // list and the picker compared keys only). No two presets may share
        // an identical table.
        // Compare rows of "key|target" - Binding itself is not Hash (it is a
        // serde DTO and stays that way), so fingerprint via Debug.
        let mut seen: HashSet<Vec<String>> = HashSet::new();
        for def in PRESETS {
            let mut table: Vec<String> = def
                .keys
                .iter()
                .map(|(k, t)| format!("{k}|{t:?}"))
                .collect();
            table.sort();
            assert!(
                seen.insert(table),
                "preset '{}' has the exact same key->target table as another preset - the picker could not tell them apart",
                def.id
            );
        }
    }

    #[test]
    fn football_presets_follow_each_games_xbox_defaults() {
        // Researched from the games' control charts / manuals:
        //  - EA FC "Classic" (its default): A pass, B shoot, X cross/lob,
        //    Y through ball, sprint on RT.
        //  - eFootball/PES "Standard" (the Konami default, unchanged since
        //    PES 2017): A low pass, X shoot, B lofted pass/cross, Y through
        //    ball, dash on RB, specials on RT. Xbox labels are positional -
        //    the PS ▢/○/△ faces map to Xbox X/B/Y.
        let at = |bs: &[Binding], key: &str| {
            bs.iter()
                .find(|b| b.key == key)
                .map(|b| b.target)
                .unwrap_or_else(|| panic!("key {key} not bound"))
        };

        let fc = bindings("football-fc").unwrap();
        assert_eq!(at(&fc, "E"), b(GamepadButton::B)); // shoot
        assert_eq!(at(&fc, "Q"), b(GamepadButton::X)); // cross / lob
        assert_eq!(at(&fc, "R"), b(GamepadButton::Y)); // through ball
        assert_eq!(at(&fc, "C"), trig(TriggerSide::Right)); // sprint (RT)

        let pes = bindings("football-pes").unwrap();
        assert_eq!(at(&pes, "E"), b(GamepadButton::X)); // shoot (Konami Standard)
        assert_eq!(at(&pes, "Q"), b(GamepadButton::B)); // lofted pass / cross
        assert_eq!(at(&pes, "R"), b(GamepadButton::Y)); // through ball
        assert_eq!(at(&pes, "F"), b(GamepadButton::RB)); // dash (RB)
        assert_eq!(at(&pes, "C"), trig(TriggerSide::Right)); // specials (RT)

        // Both keep the same ergonomic key cluster so switching games is
        // seamless; only the Xbox targets follow each game's scheme.
        let keys_of = |bs: &[Binding]| -> HashSet<String> {
            bs.iter().map(|b| b.key.clone()).collect()
        };
        assert_eq!(keys_of(&fc), keys_of(&pes));
        assert_ne!(bindings("football-fc").unwrap(), bindings("football-pes").unwrap());
    }

    #[test]
    fn every_preset_id_is_unique_and_meta_is_consistent() {
        let metas = list();
        let ids: HashSet<&str> = PRESETS.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), PRESETS.len(), "duplicate preset ids");
        assert_eq!(metas.len(), PRESETS.len());
        for (def, meta) in PRESETS.iter().zip(&metas) {
            assert_eq!(def.id, meta.id);
            assert_eq!(def.name, meta.name);
            assert_eq!(meta.key_count, def.keys.len());
            assert!(!def.description.is_empty());
            assert!(!def.keys.is_empty(), "preset {} is empty", def.id);
        }
    }

    #[test]
    fn every_key_round_trips_through_the_canonical_table() {
        // The engine compares canonical names produced from raw input events
        // against the preset key names. If name_for_vk(vk_for_name(x)) != x
        // the binding would silently never fire - catch that here for every
        // key of every preset.
        for def in PRESETS {
            for (key, _) in def.keys {
                let vk = vk_for_name(key)
                    .unwrap_or_else(|| panic!("preset {} uses unknown key '{}'", def.id, key));
                let back = name_for_vk(vk);
                assert_eq!(
                    &back, key,
                    "preset {} key '{}' is not canonical (resolves to '{back}')",
                    def.id, key
                );
            }
        }
    }

    #[test]
    fn no_key_is_bound_twice_or_reserved_in_any_preset() {
        for def in PRESETS {
            let mut seen = HashSet::new();
            for (key, _) in def.keys {
                assert!(
                    seen.insert(*key),
                    "preset {} binds key '{}' more than once",
                    def.id, key
                );
                assert!(
                    !is_os_reserved(key),
                    "preset {} uses OS-reserved key '{key}'",
                    def.id
                );
            }
        }
    }

    #[test]
    fn every_preset_has_movement_and_pause() {
        for def in PRESETS {
            let targets: Vec<Target> = def.keys.iter().map(|(_, t)| *t).collect();
            let has_movement = targets.iter().any(|t| {
                matches!(t, Target::Stick { side: StickSide::Left, .. })
            });
            assert!(has_movement, "preset {} has no left-stick movement", def.id);
            assert!(
                targets.contains(&b(GamepadButton::Start)) || def.id.contains("general"),
                "preset {} should bind Start for pausing",
                def.id
            );
        }
    }

    #[test]
    fn serde_round_trip_of_meta() {
        let metas = list();
        let json = serde_json::to_string(&metas).unwrap();
        let back: Vec<PresetMeta> = serde_json::from_str(&json).unwrap();
        assert_eq!(metas, back);
    }

    #[test]
    fn fighting_presets_use_distinct_faces_per_game() {
        // MK block = right trigger, SF6 heavy kick = trigger, Tekken has no
        // trigger requirement - spot check the researched defaults survived.
        let mk = bindings("mortal-kombat").unwrap();
        assert!(mk.iter().any(|x| x.target == trig(TriggerSide::Right)));
        let sf6 = bindings("street-fighter").unwrap();
        assert!(sf6.iter().any(|x| x.target == b(GamepadButton::RB)));
        let t7 = bindings("tekken").unwrap();
        assert!(t7.iter().all(|x| !matches!(x.target, Target::Trigger { .. })));
    }
}
