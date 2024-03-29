mod pb;
pub mod utils;

use chrono::{DateTime, Utc};
pub use pb::*;
pub use utils::*;

extern crate derive_builder;

impl Reservation {
    pub fn new_pending(
        uid: impl Into<String>,
        rid: impl Into<String>,
        note: impl Into<String>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Self {
        let mut start_timestamp = convert_to_timestamp(start);
        let mut end_timestamp = convert_to_timestamp(end);

        if start > end {
            start_timestamp = convert_to_timestamp(end);
            end_timestamp = convert_to_timestamp(start);
        }

        Reservation {
            uid: uid.into(),
            resource_id: rid.into(),
            note: note.into(),
            start: Some(start_timestamp),
            end: Some(end_timestamp),
            rstatus: 0,
            ..Default::default()
        }
    }
}

impl<T: Into<String>> From<T> for ConfirmRequest {
    fn from(value: T) -> Self {
        ConfirmRequest { id: value.into() }
    }
}

impl From<String> for CancelRequest {
    fn from(value: String) -> Self {
        CancelRequest { id: value }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_parse() {
        let datestr = "2012-03-04 05:06:07+08";
        println!("raw:{:?}", datestr);
        let date = parse_datetime(datestr);
        println!("parse:{:?}", date);
        let date = parse_datetime(datestr.strip_prefix("2012").unwrap());
        assert_eq!(date, Err(()));
    }

    #[test]
    fn test_builder() {
        let r = ReservationBuilder::default().build().unwrap();
        println!("{:?}", r);
        let r = Reservation {
            ..Default::default()
        };
        println!("{:?}", r);
    }
}
