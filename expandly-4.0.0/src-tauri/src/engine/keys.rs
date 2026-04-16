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

pub fn rkey_to_buffer_char(key: &RKey, shift_active: bool, caps_lock_on: bool) -> Option<char> {
    use RKey::*;

    let upper_alpha = shift_active ^ caps_lock_on;

    match key {
        KeyA => Some(if upper_alpha { 'A' } else { 'a' }),
        KeyB => Some(if upper_alpha { 'B' } else { 'b' }),
        KeyC => Some(if upper_alpha { 'C' } else { 'c' }),
        KeyD => Some(if upper_alpha { 'D' } else { 'd' }),
        KeyE => Some(if upper_alpha { 'E' } else { 'e' }),
        KeyF => Some(if upper_alpha { 'F' } else { 'f' }),
        KeyG => Some(if upper_alpha { 'G' } else { 'g' }),
        KeyH => Some(if upper_alpha { 'H' } else { 'h' }),
        KeyI => Some(if upper_alpha { 'I' } else { 'i' }),
        KeyJ => Some(if upper_alpha { 'J' } else { 'j' }),
        KeyK => Some(if upper_alpha { 'K' } else { 'k' }),
        KeyL => Some(if upper_alpha { 'L' } else { 'l' }),
        KeyM => Some(if upper_alpha { 'M' } else { 'm' }),
        KeyN => Some(if upper_alpha { 'N' } else { 'n' }),
        KeyO => Some(if upper_alpha { 'O' } else { 'o' }),
        KeyP => Some(if upper_alpha { 'P' } else { 'p' }),
        KeyQ => Some(if upper_alpha { 'Q' } else { 'q' }),
        KeyR => Some(if upper_alpha { 'R' } else { 'r' }),
        KeyS => Some(if upper_alpha { 'S' } else { 's' }),
        KeyT => Some(if upper_alpha { 'T' } else { 't' }),
        KeyU => Some(if upper_alpha { 'U' } else { 'u' }),
        KeyV => Some(if upper_alpha { 'V' } else { 'v' }),
        KeyW => Some(if upper_alpha { 'W' } else { 'w' }),
        KeyX => Some(if upper_alpha { 'X' } else { 'x' }),
        KeyY => Some(if upper_alpha { 'Y' } else { 'y' }),
        KeyZ => Some(if upper_alpha { 'Z' } else { 'z' }),
        Num1 => Some(if shift_active { '!' } else { '1' }),
        Num2 => Some(if shift_active { '@' } else { '2' }),
        Num3 => Some(if shift_active { '#' } else { '3' }),
        Num4 => Some(if shift_active { '$' } else { '4' }),
        Num5 => Some(if shift_active { '%' } else { '5' }),
        Num6 => Some(if shift_active { '^' } else { '6' }),
        Num7 => Some(if shift_active { '&' } else { '7' }),
        Num8 => Some(if shift_active { '*' } else { '8' }),
        Num9 => Some(if shift_active { '(' } else { '9' }),
        Num0 => Some(if shift_active { ')' } else { '0' }),
        Slash => Some(if shift_active { '?' } else { '/' }),
        BackSlash => Some(if shift_active { '|' } else { '\\' }),
        Dot => Some(if shift_active { '>' } else { '.' }),
        Comma => Some(if shift_active { '<' } else { ',' }),
        SemiColon => Some(if shift_active { ':' } else { ';' }),
        Quote => Some(if shift_active { '"' } else { '\'' }),
        LeftBracket => Some(if shift_active { '{' } else { '[' }),
        RightBracket => Some(if shift_active { '}' } else { ']' }),
        Minus => Some(if shift_active { '_' } else { '-' }),
        Equal => Some(if shift_active { '+' } else { '=' }),
        BackQuote => Some(if shift_active { '~' } else { '`' }),
        Space => Some(' '),
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
