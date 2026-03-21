use rdev::Key as RKey;

pub fn rkey_to_char(key: &RKey) -> Option<char> {
    match key {
        RKey::KeyA => Some('a'), RKey::KeyB => Some('b'), RKey::KeyC => Some('c'),
        RKey::KeyD => Some('d'), RKey::KeyE => Some('e'), RKey::KeyF => Some('f'),
        RKey::KeyG => Some('g'), RKey::KeyH => Some('h'), RKey::KeyI => Some('i'),
        RKey::KeyJ => Some('j'), RKey::KeyK => Some('k'), RKey::KeyL => Some('l'),
        RKey::KeyM => Some('m'), RKey::KeyN => Some('n'), RKey::KeyO => Some('o'),
        RKey::KeyP => Some('p'), RKey::KeyQ => Some('q'), RKey::KeyR => Some('r'),
        RKey::KeyS => Some('s'), RKey::KeyT => Some('t'), RKey::KeyU => Some('u'),
        RKey::KeyV => Some('v'), RKey::KeyW => Some('w'), RKey::KeyX => Some('x'),
        RKey::KeyY => Some('y'), RKey::KeyZ => Some('z'),
        RKey::Num0 => Some('0'), RKey::Num1 => Some('1'), RKey::Num2 => Some('2'),
        RKey::Num3 => Some('3'), RKey::Num4 => Some('4'), RKey::Num5 => Some('5'),
        RKey::Num6 => Some('6'), RKey::Num7 => Some('7'), RKey::Num8 => Some('8'),
        RKey::Num9 => Some('9'),
        RKey::Slash        => Some('/'), RKey::BackSlash    => Some('\\'),
        RKey::Dot          => Some('.'), RKey::Comma        => Some(','),
        RKey::SemiColon    => Some(';'), RKey::Quote        => Some('\''),
        RKey::LeftBracket  => Some('['), RKey::RightBracket => Some(']'),
        RKey::Minus        => Some('-'), RKey::Equal        => Some('='),
        RKey::BackQuote    => Some('`'), RKey::Space        => Some(' '),
        _ => None,
    }
}

pub fn rkey_to_modifier_str(key: &RKey) -> Option<&'static str> {
    match key {
        RKey::ControlLeft | RKey::ControlRight => Some("Control"),
        RKey::ShiftLeft   | RKey::ShiftRight   => Some("Shift"),
        RKey::Alt         | RKey::AltGr        => Some("Alt"),
        RKey::MetaLeft    | RKey::MetaRight    => Some("Super"),
        _ => None,
    }
}

pub fn rkey_to_name(key: &RKey) -> Option<String> {
    let s = match key {
        RKey::F1  => "F1",  RKey::F2  => "F2",  RKey::F3  => "F3",
        RKey::F4  => "F4",  RKey::F5  => "F5",  RKey::F6  => "F6",
        RKey::F7  => "F7",  RKey::F8  => "F8",  RKey::F9  => "F9",
        RKey::F10 => "F10", RKey::F11 => "F11", RKey::F12 => "F12",
        RKey::Tab        => "Tab",    RKey::Escape     => "Escape",
        RKey::Return     => "Return", RKey::Space      => "Space",
        RKey::UpArrow    => "Up",     RKey::DownArrow  => "Down",
        RKey::LeftArrow  => "Left",   RKey::RightArrow => "Right",
        RKey::Home       => "Home",   RKey::End        => "End",
        RKey::PageUp     => "PageUp", RKey::PageDown   => "PageDown",
        RKey::Insert     => "Insert", RKey::Delete     => "Delete",
        _ => return rkey_to_char(key).map(|c| c.to_uppercase().to_string()),
    };
    Some(s.to_string())
}
