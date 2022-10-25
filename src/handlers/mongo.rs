use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    options::{ClientOptions, FindOneAndUpdateOptions, ReturnDocument},
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

    pub async fn find_event_by_id(
        &self,
        event_id_str: &str,
    ) -> Result<Option<EventDocument>, mongodb::error::Error> {
        let event_id = ObjectId::parse_str(event_id_str).unwrap();
        let filter = doc! {"_id": event_id};
        self.events.find_one(filter, None).await
    }

    pub async fn add_event_to_day(
        &self,
        owner_str: &str,
        day: &str,
        event_id: ObjectId,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let owner_id = ObjectId::parse_str(owner_str).unwrap();
        let filter = doc! {"owner": owner_id, "day": day};
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

    pub async fn add_bookingdetail_to_event(
        &self,
        event_id: ObjectId,
        booking_detail: BookingDetail,
        duration_booked: f32,
        fully_booked: bool,
    ) -> Result<Option<EventDocument>, mongodb::error::Error> {
        let filter = doc! {"_id": event_id};
        let update_opts = doc! {
            "$set": {
                "booked": fully_booked,
                "durationBooked": duration_booked,
                "updatedAt": DateTime::now()
            },
            "$addToSet" :{
                 "bookingDetails": bson::to_bson(&booking_detail).unwrap() // Custom types need to be manually converted to BSON https://stackoverflow.com/questions/67040094/save-nested-struct-with-rust-mongodb-returns-error-the-trait-fromt-is-not-im
            },
        };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            // .upsert(true)
            .build();
        self.events
            .find_one_and_update(filter, update_opts, options)
            .await
    }

    pub async fn find_bookingdetail_by_id(
        &self,
        booking_id_str: &str,
    ) -> Result<Option<EventDocument>, mongodb::error::Error> {
        let booking_id = ObjectId::parse_str(booking_id_str).unwrap();
        let filter = doc! {"bookingDetails._id": booking_id};
        self.events.find_one(filter, None).await
    }

    pub async fn remove_bookingdetail_from_event(
        &self,
        event_id: ObjectId,
        booking_detail: &BookingDetail,
        duration_booked: f32,
    ) -> Result<Option<EventDocument>, mongodb::error::Error> {
        let filter = doc! {"_id": event_id};
        let update_opts = doc! {
            "$set": {
                "booked": false,
                "durationBooked": duration_booked,
                "updatedAt": DateTime::now()
            },
            "$pull" :{
                 "bookingDetails": bson::to_bson(&booking_detail).unwrap() // Custom types need to be manually converted to BSON https://stackoverflow.com/questions/67040094/save-nested-struct-with-rust-mongodb-returns-error-the-trait-fromt-is-not-im
            },
        };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();
        self.events
            .find_one_and_update(filter, update_opts, options)
            .await
    }

    pub async fn remove_event_from_day(
        &self,
        owner_str: &str,
        day: &String,
        event_id: ObjectId,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let owner_id = ObjectId::parse_str(owner_str).unwrap();
        let filter = doc! {"day": day, "owner": owner_id };
        let update_opts = doc! {
            "$pull" :{
                "events": event_id
            },
            "$set": {
                "updatedAt": DateTime::now()
            }
        };
        self.days.update_one(filter, update_opts, None).await
    }
}
