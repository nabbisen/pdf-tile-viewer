use super::*;

fn ch(s: &str) -> Key {
    Key::Character(s.to_string())
}

#[test]
fn plain_keys_map_to_descriptors() {
    assert_eq!(descriptor_from(&Key::Escape, false), KeyDescriptor::Escape);
    assert_eq!(
        descriptor_from(&Key::ArrowLeft, false),
        KeyDescriptor::ArrowLeft
    );
    assert_eq!(
        descriptor_from(&Key::ArrowRight, false),
        KeyDescriptor::ArrowRight
    );
    assert_eq!(descriptor_from(&Key::PageUp, false), KeyDescriptor::PageUp);
    assert_eq!(
        descriptor_from(&Key::PageDown, false),
        KeyDescriptor::PageDown
    );
    assert_eq!(descriptor_from(&Key::Home, false), KeyDescriptor::Home);
    assert_eq!(descriptor_from(&Key::End, false), KeyDescriptor::End);
    assert_eq!(
        descriptor_from(&ch("z"), false),
        KeyDescriptor::Character('z')
    );
    assert_eq!(
        descriptor_from(&ch("Z"), false),
        KeyDescriptor::Character('Z')
    );
    assert_eq!(
        descriptor_from(&ch("+"), false),
        KeyDescriptor::Character('+')
    );
    assert_eq!(
        descriptor_from(&ch(" "), false),
        KeyDescriptor::Character(' ')
    );
}

#[test]
fn keys_with_ctrl_alt_or_meta_are_not_ours() {
    for key in [Key::Escape, Key::Home, ch("z"), ch("+"), ch("0")] {
        assert_eq!(descriptor_from(&key, true), KeyDescriptor::Other, "{key:?}");
    }
}

#[test]
fn other_keys_and_multi_character_input_are_other() {
    assert_eq!(descriptor_from(&Key::Enter, false), KeyDescriptor::Other);
    assert_eq!(descriptor_from(&Key::Tab, false), KeyDescriptor::Other);
    assert_eq!(descriptor_from(&Key::F5, false), KeyDescriptor::Other);
    assert_eq!(descriptor_from(&ch("ab"), false), KeyDescriptor::Other);
    assert_eq!(descriptor_from(&ch(""), false), KeyDescriptor::Other);
}

#[test]
fn editable_fields_swallow_every_shortcut_character() {
    for c in ["z", "Z", "0", "-", "+", "=", "a", " "] {
        assert_eq!(editable_key(&ch(c)), EditableKey::Swallow, "{c:?}");
    }
    for key in [
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::Home,
        Key::End,
        Key::PageUp,
        Key::PageDown,
        Key::Backspace,
    ] {
        assert_eq!(editable_key(&key), EditableKey::Swallow, "{key:?}");
    }
}

#[test]
fn editable_fields_forward_only_escape_and_enter() {
    assert_eq!(editable_key(&Key::Escape), EditableKey::Escape);
    assert_eq!(editable_key(&Key::Enter), EditableKey::Enter);
}

#[test]
fn tab_is_left_to_bubble_for_the_modal_focus_trap() {
    assert_eq!(editable_key(&Key::Tab), EditableKey::PassThrough);
}
