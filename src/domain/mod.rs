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

pub fn parse_hex_color(input: &str) -> Option<(u8, u8, u8)> {
    let s = input.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

pub const PROJECT_PALETTE: &[(&str, &str)] = &[
    ("Blue", "#3B82F6"),
    ("Emerald", "#10B981"),
    ("Violet", "#8B5CF6"),
    ("Amber", "#F59E0B"),
    ("Rose", "#F43F5E"),
    ("Cyan", "#06B6D4"),
    ("Orange", "#F97316"),
    ("Slate", "#64748B"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_minutes() {
        assert_eq!(format_minutes(0), "0:00");
        assert_eq!(format_minutes(90), "1:30");
        assert_eq!(format_minutes(60), "1:00");
        assert_eq!(format_minutes(-45), "-0:45");
    }

    #[test]
    fn parses_supported_duration_formats() {
        assert_eq!(parse_duration_to_minutes("90"), Some(90));
        assert_eq!(parse_duration_to_minutes("1.5"), Some(90));
        assert_eq!(parse_duration_to_minutes("1:30"), Some(90));
        assert_eq!(parse_duration_to_minutes("1h30m"), Some(90));
        assert_eq!(parse_duration_to_minutes("45m"), Some(45));
        assert_eq!(parse_duration_to_minutes("2h"), Some(120));
        assert_eq!(parse_duration_to_minutes(""), None);
        assert_eq!(parse_duration_to_minutes("abc"), None);
    }

    #[test]
    fn computes_duration_from_times() {
        assert_eq!(duration_minutes_from_times("09:00", "10:30"), Some(90));
        assert_eq!(duration_minutes_from_times("23:30", "00:30"), Some(60));
        assert_eq!(duration_minutes_from_times("09:00", "09:00"), Some(0));
        assert_eq!(duration_minutes_from_times("bad", "10:00"), None);
        assert_eq!(duration_minutes_from_times("09:00", ""), None);
    }

    #[test]
    fn parses_times() {
        assert!(parse_time_hhmm("09:05").is_some());
        assert!(parse_time_hhmm("23:59").is_some());
        assert!(parse_time_hhmm("24:00").is_none());
        assert!(parse_time_hhmm("").is_none());
    }

    #[test]
    fn parses_hex_colors() {
        assert_eq!(parse_hex_color("#3B82F6"), Some((0x3B, 0x82, 0xF6)));
        assert_eq!(parse_hex_color("10B981"), Some((0x10, 0xB9, 0x81)));
        assert_eq!(parse_hex_color("invalid"), None);
        assert_eq!(parse_hex_color("#123"), None);
    }
}
