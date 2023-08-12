mod pb;
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
pub use pb::*;
use prost_types::Timestamp;

pub fn convert_to_datetime(ts: Timestamp) -> DateTime<FixedOffset> {
    DateTime::<FixedOffset>::from_local(
        NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as _).unwrap(),
        FixedOffset::east_opt(0).unwrap(),
    )
}

pub fn convert_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_micros() as _,
    }
}

#[cfg(test)]
mod tests {}
