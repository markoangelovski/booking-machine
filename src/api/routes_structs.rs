use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct EventResPayload<T> {
    pub message: String,
    pub event: Option<T>,
}

impl<T> EventResPayload<T> {
    pub fn new(message: String, event: Option<T>) -> Self {
        Self { message, event }
    }
}

#[derive(Serialize)]
pub struct ErrorResPayload {
    pub message: String,
    pub error: String,
}

impl ErrorResPayload {
    pub fn new(message: String, error: String) -> Self {
        Self { message, error }
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BookingPayload {
    pub eventId: String,
    pub day: String,
    pub amount: String,
}

impl BookingPayload {
    pub fn validate(&self) -> bool {
        let event_id_ok = self.eventId.len() == 24;
        let day_format_ok = match NaiveDate::parse_from_str(&self.day, "%Y-%m-%d") {
            Ok(_) => true,
            Err(_) => false,
        };
        // let day_format_ok = self.day.split("-").collect::<Vec<&str>>().len() == 3;
        let amount_ok = self.amount.parse::<f32>().unwrap_or_default() >= 0.25;
        if !event_id_ok || !day_format_ok || !amount_ok {
            return false;
        }
        true
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteBookingPayload {
    pub bookingId: String,
}

impl DeleteBookingPayload {
    pub fn validate(&self) -> bool {
        self.bookingId.len() == 24
    }
}
