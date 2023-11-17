use chrono::{DateTime, Utc};
use serde::{Serialize, Serializer};

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
