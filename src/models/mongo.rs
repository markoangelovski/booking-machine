use mongodb::bson::{
    oid::ObjectId, /* serde_helpers::bson_datetime_as_rfc3339_string, */ DateTime,
};
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Day {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub events: Vec<ObjectId>,
    // #[serde(with = "bson_datetime_as_rfc3339_string")]
    pub updatedAt: DateTime,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct EventDocument {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub title: String,
    pub date: f64,
    pub logs: Vec<Log>,
    pub booked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookingDetails: Option<Vec<BookingDetail>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub durationBooked: Option<f32>,
    pub day: ObjectId,
    pub duration: f32,
    // #[serde(with = "bson_datetime_as_rfc3339_string")]
    pub updatedAt: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    duration: f32,
    title: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BookingDetail {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub toDate: String,
    pub amount: f32,
}

impl BookingDetail {
    pub fn new(to_date: String, amount: f32) -> Self {
        BookingDetail {
            id: ObjectId::new(),
            toDate: to_date,
            amount,
        }
    }
}
