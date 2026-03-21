use enigo::{Direction::Click, Enigo, Key, Keyboard, Settings};

pub fn with_enigo<F: FnOnce(&mut Enigo)>(f: F) {
    match Enigo::new(&Settings::default()) {
        Ok(mut e) => f(&mut e),
        Err(e)    => eprintln!("[engine] Enigo init failed: {e}"),
    }
}

pub fn inject_text(text: &str) {
    with_enigo(|e| {
        if let Err(err) = e.text(text) {
            eprintln!("[engine] inject_text: {err}");
        }
    });
}

pub fn delete_chars(n: usize) {
    with_enigo(|e| {
        for _ in 0..n {
            let _ = e.key(Key::Backspace, Click);
        }
    });
}
