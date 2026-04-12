use rdev::Key as RKey;

pub fn rkey_to_char(key: &RKey) -> Option<char> {
    use RKey::*;

    match key {
        KeyA => Some('a'), KeyB => Some('b'), KeyC => Some('c'),
        KeyD => Some('d'), KeyE => Some('e'), KeyF => Some('f'),
        KeyG => Some('g'), KeyH => Some('h'), KeyI => Some('i'),
        KeyJ => Some('j'), KeyK => Some('k'), KeyL => Some('l'),
        KeyM => Some('m'), KeyN => Some('n'), KeyO => Some('o'),
        KeyP => Some('p'), KeyQ => Some('q'), KeyR => Some('r'),
        KeyS => Some('s'), KeyT => Some('t'), KeyU => Some('u'),
        KeyV => Some('v'), KeyW => Some('w'), KeyX => Some('x'),
        KeyY => Some('y'), KeyZ => Some('z'),
        Num0 => Some('0'), Num1 => Some('1'), Num2 => Some('2'),
        Num3 => Some('3'), Num4 => Some('4'), Num5 => Some('5'),
        Num6 => Some('6'), Num7 => Some('7'), Num8 => Some('8'),
        Num9 => Some('9'),
        Slash => Some('/'), BackSlash => Some('\\'),
        Dot => Some('.'), Comma => Some(','),
        SemiColon => Some(';'), Quote => Some('\''),
        LeftBracket => Some('['), RightBracket => Some(']'),
        Minus => Some('-'), Equal => Some('='),
        BackQuote => Some('`'), Space => Some(' '),
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
