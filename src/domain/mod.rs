use chrono::{NaiveTime, Timelike};

pub fn format_minutes(total: i64) -> String {
    let sign = if total < 0 { "-" } else { "" };
    let t = total.abs();
    let h = t / 60;
    let m = t % 60;
    format!("{}{}:{:02}", sign, h, m)
}

pub fn parse_time_hhmm(input: &str) -> Option<NaiveTime> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    NaiveTime::parse_from_str(s, "%H:%M")
        .ok()
        .or_else(|| NaiveTime::parse_from_str(s, "%H:%M:%S").ok())
}

pub fn duration_minutes_from_times(start: &str, end: &str) -> Option<i64> {
    let s = parse_time_hhmm(start)?;
    let e = parse_time_hhmm(end)?;
    let start_m = s.hour() as i64 * 60 + s.minute() as i64;
    let mut end_m = e.hour() as i64 * 60 + e.minute() as i64;
    if end_m < start_m {
        // assume crossed midnight
        end_m += 24 * 60;
    }
    Some(end_m - start_m)
}

pub fn now_hhmm() -> String {
    chrono::Local::now().format("%H:%M").to_string()
}

pub fn parse_duration_to_minutes(input: &str) -> Option<i64> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    // formats: "90", "1.5", "1:30", "1h30m", "1h", "30m"
    if let Ok(mins) = s.parse::<i64>() {
        return Some(mins);
    }
    if let Ok(hours) = s.parse::<f64>() {
        return Some((hours * 60.0).round() as i64);
    }
    if s.contains(':') {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 2 {
            let h: i64 = parts[0].trim().parse().ok()?;
            let m: i64 = parts[1].trim().parse().ok()?;
            return Some(h * 60 + m);
        }
    }
    let lower = s.to_lowercase();
    if lower.contains('h') || lower.contains('m') {
        let mut h: i64 = 0;
        let mut m: i64 = 0;
        let mut num = String::new();
        for ch in lower.chars() {
            if ch.is_ascii_digit() {
                num.push(ch);
            } else if ch == 'h' {
                if !num.is_empty() {
                    h = num.parse().ok()?;
                    num.clear();
                }
            } else if ch == 'm' {
                if !num.is_empty() {
                    m = num.parse().ok()?;
                    num.clear();
                }
            }
        }
        return Some(h * 60 + m);
    }
    None
}
