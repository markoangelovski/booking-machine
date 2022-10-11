use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Response {
    pub message: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BookingPayload {
    pub eventId: String,
    pub day: String,
    pub amount: String,
}
