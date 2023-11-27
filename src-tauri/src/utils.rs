use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Serialize, Serializer};

pub fn datetime_to_filename(dt: DateTime<Utc>) -> String {
    // Formats the datetime in a filename-friendly format
    // Example: 2023-03-15T123456Z
    dt.to_rfc3339_opts(SecondsFormat::Secs, true)
        .replace(":", "")
}

pub fn parse_rgb(rgb_str: &str) -> Result<[u8; 3], String> {
    // Trim the leading 'rgb(' and trailing ')'
    let trimmed = rgb_str.trim_start_matches("rgb(").trim_end_matches(")");

    // Split the string by commas
    let parts: Vec<&str> = trimmed.split(',').collect();

    if parts.len() != 3 {
        return Err("Invalid RGB format".to_string());
    }

    // Parse each part as an integer
    let r = parts[0]
        .trim()
        .parse::<u8>()
        .map_err(|_| "Invalid red value")?;
    let g = parts[1]
        .trim()
        .parse::<u8>()
        .map_err(|_| "Invalid green value")?;
    let b = parts[2]
        .trim()
        .parse::<u8>()
        .map_err(|_| "Invalid blue value")?;

    Ok([r, g, b])
}

pub fn serialize_dt<S>(dt: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(dt) = dt {
        dt.format("%m/%d/%Y %H:%M")
            .to_string()
            .serialize(serializer)
    } else {
        serializer.serialize_none()
    }
}
