use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use prost_types::Timestamp;

pub fn convert_to_datetime(ts: Timestamp) -> DateTime<FixedOffset> {
    DateTime::<FixedOffset>::from_naive_utc_and_offset(
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

pub struct DateTimeOffset(pub DateTime<FixedOffset>);

impl From<DateTimeOffset> for Timestamp {
    fn from(val: DateTimeOffset) -> Self {
        Timestamp {
            seconds: val.0.timestamp(),
            nanos: val.0.timestamp_subsec_micros() as _,
        }
    }
}

#[allow(clippy::result_unit_err)]
pub fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ()> {
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%#z")
        .map_err(|_| {
            println!(" parse_datetime:{:?} ERROR", s);
        })?
        .with_timezone(&Utc))
}

#[allow(unused_macros)]
macro_rules! printlnn {
    ($($arg:tt),+)=>{
        $(println!($arg);)+
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn macro_test() {
        printlnn!("1", "2", "3", "4", "5", "6", "7");
    }
}
