use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, extjson::de::Error, oid::ObjectId, DateTime},
    options::ClientOptions,
    options::FindOptions,
    results::UpdateResult,
    Client, Collection,
};

use crate::models::mongo::{BookingDetail, Day, EventDocument};

pub struct MongoDB {
    days: Collection<Day>,
    events: Collection<EventDocument>,
}

impl MongoDB {
    pub async fn init() -> Self {
        let uri = if std::env::var("ENV").unwrap_or("development".to_string()) == "development" {
            match std::env::var("MONGO_URI_DEV") {
                Ok(uri) => uri,
                Err(_) => format!("Error getting Dev Mongo URI!"),
            }
        } else {
            match std::env::var("MONGO_URI") {
                Ok(uri) => uri,
                Err(_) => format!("Error getting Mongo URI!"),
            }
        };
        let client_options = ClientOptions::parse(uri).await.unwrap();
        let host = client_options.hosts[0].clone();
        let client = Client::with_options(client_options).expect("Connection to Mongo DB failed!");
        println!("Connected to the following Mongo DB host: {host}");
        let db = client.database("project-manager");
        let days: Collection<Day> = db.collection("days");
        let events: Collection<EventDocument> = db.collection("events");
        MongoDB { days, events }
    }

    pub async fn test(&self) -> Result<Vec<EventDocument>, Error> {
        let find_options = FindOptions::builder().limit(10).skip(100).build();

        let mut cursors = self
            .events
            .find(None, find_options)
            .await
            .ok()
            .expect("Error getting list of days");

        let mut days: Vec<EventDocument> = Vec::new();

        while let Some(day) = cursors
            .try_next()
            .await
            .ok()
            .expect("Error mapping through cursor")
        {
            println!("Day: {:?}", day);
            days.push(day)
        }
        Ok(days)
    }

    pub async fn find_day(
        &self,
        owner_str: &str,
        day: &str,
    ) -> Result<Option<Day>, mongodb::error::Error> {
        let owner_id = ObjectId::parse_str(owner_str).unwrap();
        let filter = doc! {"owner": owner_id, "day": day};
        self.days.find_one(filter, None).await
    }

    pub async fn find_event(
        &self,
        event_id_str: &str,
    ) -> Result<Option<EventDocument>, mongodb::error::Error> {
        let event_id = ObjectId::parse_str(event_id_str).unwrap();
        let filter = doc! {"_id": event_id};
        self.events.find_one(filter, None).await
    }

    pub async fn update_day(
        &self,
        day_id: ObjectId,
        event_id: ObjectId,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let filter = doc! {"_id": day_id};
        let update_opts = doc! {
            "$addToSet" :{
                "events": event_id
            },
            "$set": {
                "updatedAt": DateTime::now()
            }
        };
        self.days.update_one(filter, update_opts, None).await
    }

    pub async fn update_event(
        &self,
        event_id: ObjectId,
        booking_detail: BookingDetail,
        external_bookings_duration: f32,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let filter = doc! {"_id": event_id};
        let update_opts = doc! {
            "$addToSet" :{
                 "externalBookingDetails": bson::to_bson(&booking_detail).unwrap() // Custom types need to be manually converted to BSON https://stackoverflow.com/questions/67040094/save-nested-struct-with-rust-mongodb-returns-error-the-trait-fromt-is-not-im
            },
            "$set": {
                "externalBookingDuration": external_bookings_duration,
                "updatedAt": DateTime::now()
            }
        };
        self.events.update_one(filter, update_opts, None).await
    }
}
