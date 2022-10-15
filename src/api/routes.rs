use actix_web::{
    get, post,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use futures::join;
use mongodb::bson::oid::ObjectId;

use super::routes_structs::{BookingPayload, StdRes};

use crate::handlers::mongo::MongoDB;
use crate::middlewares::auth::UserId;
use crate::models::mongo::BookingDetail;

#[get("/days")]
pub async fn test(db: Data<MongoDB>) -> HttpResponse {
    match db.test().await {
        Ok(days) => HttpResponse::Ok().json(days),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[post("/book")]
pub async fn book_event(
    db: Data<MongoDB>,
    query: Query<BookingPayload>,
    user_id: ReqData<UserId>,
) -> HttpResponse {
    println!("User ID{:?}", user_id);

    let UserId(user_id) = user_id.into_inner();

    let amount: f32 = query.amount.parse().unwrap_or_default();

    let (day, event) = join!(
        db.find_day(&user_id, &query.day),
        db.find_event(&query.eventId)
    );

    let day = match day {
        Ok(day) => match day {
            Some(day_doc) => day_doc,
            None => {
                return HttpResponse::NotFound().json(StdRes {
                    message: "Day not found".to_string(),
                })
            }
        },
        Err(err) => {
            return HttpResponse::InternalServerError().json(StdRes {
                message: err.to_string(),
            })
        }
    };

    let event = match event {
        Ok(event) => match event {
            Some(event_doc) => event_doc,
            None => {
                return HttpResponse::NotFound().json(StdRes {
                    message: "Event not found".to_string(),
                })
            }
        },
        Err(err) => {
            return HttpResponse::InternalServerError().json(StdRes {
                message: err.to_string(),
            })
        }
    };

    let booking_detail = BookingDetail {
        _id: ObjectId::new(),
        toDay: day.id,
        toDate: query.day.to_string(),
        amount,
    };

    let external_bookings_duration = if !event.externalBookingDetails.is_none() {
        event
            .externalBookingDetails
            .unwrap()
            .iter()
            .fold(0.0, |acc, booking_detail| acc + booking_detail.amount)
            + amount
    } else {
        amount
    };

    let (updated_day_result, updated_event_result) = join!(
        db.update_event(
            event.id.unwrap(),
            booking_detail,
            external_bookings_duration,
        ),
        db.update_day(day.id, event.id.unwrap())
    );

    match updated_day_result {
        Ok(result) => result,
        Err(err) => {
            return HttpResponse::InternalServerError().json(StdRes {
                message: err.to_string(),
            })
        }
    };

    match updated_event_result {
        Ok(result) => result,
        Err(err) => {
            return HttpResponse::InternalServerError().json(StdRes {
                message: err.to_string(),
            })
        }
    };

    HttpResponse::Ok().json(StdRes {
        message: "ok".to_string(),
    })
}
