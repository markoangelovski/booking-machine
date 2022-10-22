use chrono::{NaiveDate, NaiveDateTime};

pub fn compare(timestamp: f64, datestring: &String) -> bool {
    let event_date = NaiveDateTime::from_timestamp_opt(timestamp as i64 / 1000, 0)
        .expect("Invalid Event date format")
        .format("%Y-%m-%d")
        .to_string();

    let booking_detail_date = NaiveDate::parse_from_str(&datestring, "%Y-%m-%d")
        .expect("Invalid Booking Detail date format")
        .and_hms(0, 0, 0)
        .format("%Y-%m-%d")
        .to_string();

    event_date == booking_detail_date
}
