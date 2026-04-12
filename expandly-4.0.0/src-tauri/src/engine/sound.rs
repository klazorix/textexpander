use std::{thread, time::Duration};

fn is_remote_path(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://")
}

pub fn play_sound(path: String) {
    thread::spawn(move || {
        use rodio::{Decoder, OutputStream, Sink};
        use std::io::BufReader;

        let Ok((_stream, handle)) = OutputStream::try_default() else { return };
        let Ok(sink)              = Sink::try_new(&handle)       else { return };

        if is_remote_path(&path) {
            use std::io::{Cursor, Read};
            let Ok(response) = ureq::get(&path).call() else { return };
            let mut bytes = Vec::new();
            let Ok(_) = response.into_reader().read_to_end(&mut bytes) else { return };
            let cursor = Cursor::new(bytes);
            let Ok(source) = Decoder::new(BufReader::new(cursor)) else { return };
            sink.append(source);
        } else {
            use std::fs::File;
            let Ok(file)   = File::open(&path)                  else { return };
            let Ok(source) = Decoder::new(BufReader::new(file)) else { return };
            sink.append(source);
        }

        let t = std::time::Instant::now();
        while !sink.empty() {
            if t.elapsed().as_secs() >= 10 { sink.stop(); break; }
            thread::sleep(Duration::from_millis(100));
        }
    });
}
