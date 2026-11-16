pub fn format_minutes(total: i64) -> String {
    let sign = if total < 0 { "-" } else { "" };
    let t = total.abs();
    let h = t / 60;
    let m = t % 60;
    format!("{}{}:{:02}", sign, h, m)
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
