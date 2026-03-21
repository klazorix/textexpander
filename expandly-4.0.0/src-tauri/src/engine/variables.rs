use crate::models::RootConfig;

pub struct DateTime {
    pub date: String,
    pub time: String,
    pub day: String,
    pub month: String,
    pub year: String,
}

pub fn days_from_epoch(z: i64) -> (i64, i64, i64) {
    let z   = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y   = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp  = (5 * doy + 2) / 153;
    let d   = doy - (153 * mp + 2) / 5 + 1;
    let m   = if mp < 10 { mp + 3 } else { mp - 9 };
    let y   = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

pub fn chrono_now() -> DateTime {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs             = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let days_since_epoch = secs / 86400;
    let secs_today       = secs % 86400;
    let (y, m, d)        = days_from_epoch(days_since_epoch as i64);
    let dow              = ((days_since_epoch + 3) % 7) as usize;
    let days   = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    let months = ["January", "February", "March", "April", "May", "June",
                  "July", "August", "September", "October", "November", "December"];
    DateTime {
        date:  format!("{:02}/{:02}/{}", d, m, y),
        time:  format!("{:02}:{:02}", secs_today / 3600, (secs_today % 3600) / 60),
        day:   days[dow].to_string(),
        month: months[(m - 1) as usize].to_string(),
        year:  y.to_string(),
    }
}

pub fn today_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs      = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let (y, m, d) = days_from_epoch((secs / 86400) as i64);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

pub fn get_clipboard() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let out = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", "Get-Clipboard"])
            .output()
            .ok()?;
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }
    #[cfg(not(target_os = "windows"))]
    { None }
}

pub fn resolve_variables(text: &str, config: &RootConfig) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs             = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let days_since_epoch = secs / 86400;
    let secs_today       = secs % 86400;

    let now = chrono_now();
    let mut result = text.to_string();

    result = result.replace("{date}",     &now.date);
    result = result.replace("{time}",     &now.time);
    result = result.replace("{datetime}", &format!("{} {}", now.date, now.time));
    result = result.replace("{day}",      &now.day);
    result = result.replace("{month}",    &now.month);
    result = result.replace("{year}",     &now.year);
    result = result.replace("{hour}",     &format!("{:02}", secs_today / 3600));
    result = result.replace("{minute}",   &format!("{:02}", (secs_today % 3600) / 60));

    let yesterday = { let (y, m, d) = days_from_epoch(days_since_epoch as i64 - 1); format!("{:02}/{:02}/{}", d, m, y) };
    let tomorrow  = { let (y, m, d) = days_from_epoch(days_since_epoch as i64 + 1); format!("{:02}/{:02}/{}", d, m, y) };
    result = result.replace("{yesterday}", &yesterday);
    result = result.replace("{tomorrow}",  &tomorrow);

    let greeting = match secs_today / 3600 {
        5..=11  => "Good morning",
        12..=17 => "Good afternoon",
        18..=21 => "Good evening",
        _       => "Good night",
    };
    result = result.replace("{greeting}", greeting);

    if result.contains("{clipboard}") {
        result = result.replace("{clipboard}", &get_clipboard().unwrap_or_default());
    }

    for var in &config.custom_variables {
        result = result.replace(&format!("{{{}}}", var.name), &var.value);
    }

    result
}
