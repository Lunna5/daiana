//! Custom error types and HTTP response mappings.

use actix_web::HttpResponse;
use actix_web::body::BoxBody;
use actix_web::http::{StatusCode, header};
use derive_more::{Display, Error};
use serde_json::json;

/// Application errors that can occur during room management and WebSocket handling.
#[derive(Debug, Display, Error)]
pub enum DaianaError {
    /// An unexpected internal server error occurred.
    #[display("internal error")]
    InternalError,

    /// An invalid argument was supplied.
    #[display("invalid argument")]
    InvalidArgument,

    /// An operation timed out.
    #[display("timeout")]
    Timeout,

    /// The specified room UUID does not exist or is invalid.
    #[display("room not found or invalid")]
    InvalidRoomId,

    /// Failed to initialize or upgrade the WebSocket connection.
    #[display("something failed while initialising the websocket connection")]
    Websocket,

    /// The target room has reached its maximum client capacity.
    #[display("maximum clients on a room reached")]
    MaximumClientsReached,
}

impl actix_web::error::ResponseError for DaianaError {
    fn status_code(&self) -> StatusCode {
        match *self {
            DaianaError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            DaianaError::InvalidArgument => StatusCode::BAD_REQUEST,
            DaianaError::Timeout => StatusCode::GATEWAY_TIMEOUT,
            DaianaError::InvalidRoomId => StatusCode::NOT_FOUND,
            DaianaError::Websocket => StatusCode::BAD_REQUEST,
            DaianaError::MaximumClientsReached => StatusCode::CONFLICT,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(header::ContentType::json())
            .json(json!({ "error": self.to_string() }))
    }
}
