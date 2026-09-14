use std::str::FromStr;

use inox_platform::{
    InputState, Key, PLATFORM_TYPE_PC, PLATFORM_TYPE_PC_NAME, PLATFORM_TYPE_WEB,
    PLATFORM_TYPE_WEB_NAME,
};

#[test]
fn parses_keys_independently_of_platform() {
    assert_eq!(Key::from_str("a"), Ok(Key::A));
    assert_eq!(Key::from_str("1"), Ok(Key::Numpad1));
    assert_eq!(Key::from_str("unknown"), Err(Key::Unidentified));
}

#[test]
fn exposes_stable_platform_constants_and_input_states() {
    assert_eq!(PLATFORM_TYPE_PC, 0);
    assert_eq!(PLATFORM_TYPE_WEB, 1);
    assert_eq!(PLATFORM_TYPE_PC_NAME, "pc");
    assert_eq!(PLATFORM_TYPE_WEB_NAME, "web");
    assert!(InputState::JustPressed < InputState::Pressed);
    assert_ne!(InputState::Released, InputState::JustReleased);
}
