use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Day {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub events: Vec<ObjectId>,
    pub updatedAt: DateTime,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct EventDocument {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub externalBookingDetails: Option<Vec<BookingDetail>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub externalBookingDuration: Option<f32>,
    pub day: ObjectId,
    pub updatedAt: Option<DateTime>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct BookingDetail {
    pub _id: ObjectId,
    pub toDay: ObjectId,
    pub toDate: String,
    pub amount: f32,
}
