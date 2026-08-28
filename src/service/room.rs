use actix_web::{post, HttpResponse, Scope, get, web, Responder, HttpRequest, rt};
use actix_web::web::Data;
use actix_ws::AggregatedMessage;
use serde_json::json;
use crate::{packet, AppState};
use crate::util::error::DaianaError;
use futures_util::StreamExt as _;
use log::{debug, error};
use uuid::{Bytes, Uuid};
use crate::channel::Client;
use crate::packet::{WsInPacket, WsPacket};

#[post("/")]
async fn create_room(state: Data<AppState>) -> impl Responder {
    let channel_manager = &state.channel_manager;
    let id = channel_manager.create_channel();

    HttpResponse::Ok().json(json!({ "id": id }))
}

#[get("/{id}")]
async fn connect_ws(
    state: Data<AppState>,
    path: web::Path<(String,)>,
    req: HttpRequest,
    stream: web::Payload
) -> Result<HttpResponse, DaianaError> {
    let id = path.into_inner().0;
    let room_uuid = Uuid::parse_str(&id).map_err(|_| DaianaError::InvalidRoomId)?;

    {
        let channel_manager = &state.channel_manager;

        if !channel_manager.channel_exists(room_uuid) {
            return Err(DaianaError::InvalidRoomId);
        }
    }

    let client_uuid = Uuid::new_v4();


    debug!("User with uuid: {}, connected to room {}", &client_uuid, &room_uuid);

    let (res, mut session, stream) = actix_ws::handle(&req, stream)
        .map_err(|error| {
            error!("{}", error);
            DaianaError::Websocket {}
        })?;

    let client = Client::new(client_uuid, session.clone());

    {
        let channel_manager = &state.channel_manager;

        if !channel_manager.channel_exists(room_uuid) {
            return Err(DaianaError::InvalidRoomId);
        }

        channel_manager.insert_client(room_uuid, client)?;
    }

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20)); // Aggregate continuation frames up to 1MiB

    {
        packet::connect_and_broadcast(&state.channel_manager, room_uuid, client_uuid).await;
    }

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Binary(msg)) => {
                    match WsInPacket::from_bytes(msg) {

                        Ok(WsInPacket::Unicast { target_id, payload }) => {
                            let out_packet = WsPacket::Message {
                                sender_id: client_uuid,
                                payload
                            };

                            packet::send_to_client(&state.channel_manager, room_uuid, target_id, &out_packet).await;
                        }

                        Ok(WsInPacket::Multicast { target_ids, payload }) => {
                            let out_packet = WsPacket::Message {
                                sender_id: client_uuid,
                                payload
                            };

                            packet::multicast_to_clients(&state.channel_manager, room_uuid, &target_ids, &out_packet).await;
                        }

                        Ok(WsInPacket::Broadcast { payload }) => {
                            let out_packet = WsPacket::Message {
                                sender_id: client_uuid,
                                payload
                            };

                            Box::pin(packet::broadcast_to_room(&state.channel_manager, room_uuid, &out_packet, Some(client_uuid))).await;
                        }

                        Err(e) => {
                            error!("Error while parsing client packet {}: {:?}", client_uuid, e);
                        }
                    }
                }

                Ok(AggregatedMessage::Text(text)) => {
                    session.text("Server does not support text input").await.unwrap();
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }

                Ok(AggregatedMessage::Close(reason)) => {
                    debug!("Connection closed by {}", client_uuid);
                    break;
                }

                _ => {}
            }
        }

        {
            packet::disconnect_and_broadcast(&state.channel_manager, room_uuid, client_uuid).await;
        }
    });

    Ok(res)
}

pub fn endpoints(scope: Scope) -> Scope {
    scope.service(create_room).service(connect_ws)
}