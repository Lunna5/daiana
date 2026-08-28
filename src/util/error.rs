use actix_web::HttpResponse;
use actix_web::body::BoxBody;
use actix_web::http::{StatusCode, header};
use derive_more::{Display, Error};
use serde_json::json;

#[derive(Debug, Display, Error)]
pub enum DaianaError {
    #[display("internal error")]
    InternalError,

    #[display("invalid argument")]
    InvalidArgument,

    #[display("timeout")]
    Timeout,

    #[display("room not found or invalid")]
    InvalidRoomId,

    #[display("something failed while initialising the websocket connection")]
    Websocket,

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
