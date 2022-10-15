use std::{
    env,
    future::{ready, Ready},
};

use actix_web::{
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, errors::ErrorKind, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use urlencoding::decode as url_decode;

use crate::api::routes_structs::StdRes;

pub struct CheckLoginFactory;

impl<S, B> Transform<S, ServiceRequest> for CheckLoginFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckLoginMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CheckLoginMiddleware { service }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    _id: String,
    iat: usize,
    exp: usize,
}

#[derive(Debug, Clone)]
pub struct UserId(pub String);

pub struct CheckLoginMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CheckLoginMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let mut is_logged_in = false;

        let url_encoded_token = match request.headers().get("Authorization") {
            Some(auth_header) => match auth_header.to_str() {
                Ok(auth_header) => auth_header.to_string(),
                Err(_) => "".to_string(),
            },
            None => "".to_string(),
        };
        let decoded_token = url_decode(&url_encoded_token).unwrap_or_default();

        let token = if decoded_token.len() > 1 {
            decoded_token.split(" ").collect::<Vec<&str>>()[1]
        } else {
            ""
        };

        let key = env::var("USER_AUTH").expect("JWT User secret not set!");

        let validation = Validation::new(Algorithm::HS256);

        let token_data = match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(&key.into_bytes()),
            &validation,
        ) {
            Ok(c) => {
                is_logged_in = true;
                c.claims._id
            }
            Err(err) => match *err.kind() {
                ErrorKind::InvalidToken => "- Token is invalid".into(), // Example on how to handle a specific error
                ErrorKind::ExpiredSignature => "- Token expired".into(), // Example on how to handle a specific error
                ErrorKind::InvalidSignature => "- Token signature is invalid".into(), // Example on how to handle a specific error
                _ => "- Some other errors".into(),
            },
        };

        if !is_logged_in {
            let (request, _pl) = request.into_parts();

            let response = HttpResponse::Unauthorized()
                .json(StdRes {
                    // message: "Unauthorized".to_string(),
                    message: format!("Unauthorized {}", token_data),
                })
                // constructed responses map to "right" body, early return res to client
                .map_into_right_body();

            return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
        }

        request.extensions_mut().insert(UserId(token_data));

        let res = self.service.call(request);

        Box::pin(async move {
            // forwarded responses map to "left" body, continue with the request
            res.await.map(ServiceResponse::map_into_left_body)
        })
    }
}
