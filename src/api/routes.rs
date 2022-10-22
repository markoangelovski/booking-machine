use actix_web::{
    delete, post,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use futures::join;
use mongodb::bson::oid::ObjectId;

use super::routes_helpers::compare;
use super::routes_structs::{
    BookingPayload, DeleteBookingPayload, ErrorResPayload, EventResPayload,
};

use crate::handlers::mongo::MongoDB;
use crate::middlewares::auth::UserId;
use crate::models::mongo::BookingDetail;

#[post("/book")]
pub async fn book_event(
    db: Data<MongoDB>,
    query: Query<BookingPayload>,
    user_id: ReqData<UserId>,
) -> HttpResponse {
    let UserId(user_id) = user_id.into_inner();

    let amount: f32 = query.amount.parse().unwrap_or_default();

    // TODO: Move validation to middleware?
    if !query.validate() {
        return HttpResponse::UnprocessableEntity().json(ErrorResPayload::new(
            "An error occurred!".to_string(),
            "Invalid date format or amount. Required date format: YYYY-MM-DD. Amount must be at least 0.25h".to_string(),
        ));
    }

    let (day, event) = join!(
        db.find_day_by_datestring(&user_id, &query.day),
        db.find_event_by_id(&query.eventId)
    );
    let day = match day {
        Ok(day) => match day {
            Some(day_doc) => day_doc,
            None => {
                return HttpResponse::NotFound().json(ErrorResPayload::new(
                    "An error occurred!".to_string(),
                    "Day not found".to_string(),
                ))
            }
        },
        Err(err) => {
            return HttpResponse::InternalServerError().json(ErrorResPayload::new(
                "An error occurred while fetching the day!".to_string(),
                err.to_string(),
            ))
        }
    };

    let event = match event {
        Ok(event) => match event {
            Some(event_doc) => event_doc,
            None => {
                return HttpResponse::NotFound().json(ErrorResPayload::new(
                    "An error occurred!".to_string(),
                    "Event not found".to_string(),
                ))
            }
        },
        Err(err) => {
            return HttpResponse::InternalServerError().json(ErrorResPayload::new(
                "An error occurred while fetching the event!".to_string(),
                err.to_string(),
            ))
        }
    };

    // Construct the new BookingDetail object
    let booking_detail = BookingDetail::new(query.day.to_string(), amount);

    let mut duration_booked = 0.0;
    let mut booking_detail_exists = false;

    if event.bookingDetails.is_some() {
        let deets = event.bookingDetails.unwrap();
        // Check if BookingDetail exists
        booking_detail_exists = deets
            .iter()
            .position(|detail| detail == &booking_detail)
            .is_some();
        // Calculate duration of existing booked time
        duration_booked = deets
            .iter()
            .fold(0.0, |acc, booking_detail| acc + booking_detail.amount);
    }
    // If no duration is booked, add the submitted amount to total booking time
    if !booking_detail_exists {
        duration_booked = duration_booked + amount
    };

    // Do not allow more booking time than worked time
    if duration_booked > event.duration {
        return HttpResponse::BadRequest().json(ErrorResPayload::new(
            "An error occurred!".to_string(),
            format!(
                "Unallowed amount: {}h, available booking hours: {}h",
                amount,
                event.duration - (duration_booked - amount)
            ),
        ));
    }

    let fully_booked = duration_booked == event.duration;

    let (updated_day_result, updated_event_result) = join!(
        db.add_event_to_day(day.id, event.id),
        db.add_bookingdetail_to_event(event.id, booking_detail, duration_booked, fully_booked)
    );

    match updated_day_result {
        Ok(_) => (),
        Err(err) => {
            return HttpResponse::InternalServerError().json(ErrorResPayload::new(
                "An error occurred while updating the day!".to_string(),
                err.to_string(),
            ))
        }
    };

    let updated_event = match updated_event_result {
        Ok(event_opt) => event_opt,
        Err(err) => {
            return HttpResponse::InternalServerError().json(ErrorResPayload::new(
                "An error occurred while updating the event!".to_string(),
                err.to_string(),
            ))
        }
    };
    HttpResponse::Ok().json(EventResPayload::new(
        "Booking completed.".to_string(),
        updated_event,
    ))
}

#[delete("/delete")]
pub async fn delete_event(
    db: Data<MongoDB>,
    query: Query<DeleteBookingPayload>,
    user_id: ReqData<UserId>,
) -> HttpResponse {
    // TODO: Move validation to middleware?
    if !query.validate() {
        return HttpResponse::UnprocessableEntity().json(ErrorResPayload::new(
            "An error occurred!".to_string(),
            "Invalid ID format!".to_string(),
        ));
    }

    let UserId(user_id) = user_id.into_inner();

    let DeleteBookingPayload {
        bookingId: booking_id_str,
    } = query.into_inner();

    let event = match db.find_bookingdetail_by_id(&booking_id_str).await {
        Ok(event) => match event {
            Some(event) => event,
            None => {
                return HttpResponse::NotFound().json(ErrorResPayload::new(
                    "An error ocurred!".to_string(),
                    "Booking detail not found!".to_string(),
                ))
            }
        },
        Err(err) => {
            return HttpResponse::InternalServerError().json(ErrorResPayload::new(
                "An error ocurred!".to_string(),
                err.to_string(),
            ))
        }
    };

    let duration_booked = match &event.durationBooked {
        Some(duration_booked) => duration_booked,
        _ => unreachable!(),
    };

    let booking_details = match &event.bookingDetails {
        Some(booking_details) => booking_details,
        _ => unreachable!(),
    };

    let booking_detail = match booking_details
        .iter()
        .find(|detail| detail.id == ObjectId::parse_str(&booking_id_str).unwrap())
    {
        Some(booking_detail) => booking_detail,
        _ => unreachable!(),
    };

    let updated_duration_booked = duration_booked - booking_detail.amount;

    // Check if multiple details with the same destination date exist
    let has_more_details = booking_details
        .iter()
        .filter(|detail| detail.toDate == booking_detail.toDate)
        .collect::<Vec<&BookingDetail>>()
        .len()
        > 1;

    // Delete the Booking Detail from the Event
    let updated_event = match db
        .remove_bookingdetail_from_event(event.id, booking_detail, updated_duration_booked)
        .await
    {
        Ok(event) => match event {
            Some(event) => event,
            _ => unreachable!(),
        },
        Err(err) => {
            return HttpResponse::InternalServerError().json(ErrorResPayload::new(
                "An error ocurred!".to_string(),
                err.to_string(),
            ))
        }
    };

    // Check if the Event and Booking Detail belong to the same destination Day and if there are multiple Booking Details for the destination Day
    // Removes the event from the destination Day
    if !compare(event.date, &booking_detail.toDate) && !has_more_details {
        match db
            .remove_event_from_day(&user_id, &booking_detail.toDate, event.id)
            .await
        {
            Ok(_) => (),
            Err(err) => {
                return HttpResponse::InternalServerError().json(ErrorResPayload::new(
                    "An error ocurred".to_string(),
                    err.to_string(),
                ))
            }
        };
    }

    HttpResponse::Ok().json(EventResPayload::new(
        "Booking detail deleted!".to_string(),
        Some(updated_event),
    ))
}
