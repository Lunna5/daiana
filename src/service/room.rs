//! Room creation REST endpoint and WebSocket upgrade handlers.

use crate::channel::Client;
use crate::packet::{WsInPacket, WsPacket};
use crate::util::error::DaianaError;
use crate::{AppState, packet};
use actix_web::web::Data;
use actix_web::{HttpRequest, HttpResponse, Responder, Scope, get, post, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;
use log::{debug, error, warn};
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

/// Creates a new room via `POST /room/` and returns its generated UUID in JSON format.
#[post("/")]
async fn create_room(state: Data<AppState>) -> impl Responder {
    let channel_manager = &state.channel_manager;
    let id = channel_manager.create_channel();

    HttpResponse::Ok()
        .insert_header(("Location", format!("/room/{}", id)))
        .json(json!({ "id": id }))
}

/// Upgrades an incoming HTTP request to a WebSocket connection for the room identified by `{id}` (`GET /room/{id}`).
#[get("/{id}")]
async fn connect_ws(
    state: Data<AppState>,
    path: web::Path<(String,)>,
    req: HttpRequest,
    stream: web::Payload,
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

    debug!(
        "User with uuid: {}, connected to room {}",
        client_uuid, room_uuid
    );

    let (res, mut session, stream) = actix_ws::handle(&req, stream).map_err(|error| {
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

    let max_packets_per_sec = state.max_packets_per_sec;
    let max_packet_size_bytes = state.max_packet_size_bytes;

    rt::spawn(async move {
        let mut packet_count = 0u32;
        let mut last_reset = Instant::now();

        while let Some(msg) = stream.next().await {
            if last_reset.elapsed().as_secs() >= 1 {
                packet_count = 0;
                last_reset = Instant::now();
            }

            packet_count += 1;
            if max_packets_per_sec > 0 && packet_count > max_packets_per_sec {
                warn!(
                    "Client {} exceeded rate limit ({} pkt/s > {}). Dropping packet.",
                    client_uuid, packet_count, max_packets_per_sec
                );
                continue;
            }

            match msg {
                Ok(AggregatedMessage::Binary(msg)) => {
                    if max_packet_size_bytes > 0 && msg.len() > max_packet_size_bytes {
                        warn!(
                            "Client {} sent oversized packet ({} bytes > {}). Dropping packet.",
                            client_uuid,
                            msg.len(),
                            max_packet_size_bytes
                        );
                        continue;
                    }

                    match WsInPacket::from_bytes(msg) {
                        Ok(WsInPacket::Unicast { target_id, payload }) => {
                            let out_packet = WsPacket::Message {
                                sender_id: client_uuid,
                                payload,
                            };

                            packet::send_to_client(
                                &state.channel_manager,
                                room_uuid,
                                target_id,
                                &out_packet,
                            )
                            .await;
                        }

                        Ok(WsInPacket::Multicast {
                            target_ids,
                            payload,
                        }) => {
                            let out_packet = WsPacket::Message {
                                sender_id: client_uuid,
                                payload,
                            };

                            packet::multicast_to_clients(
                                &state.channel_manager,
                                room_uuid,
                                &target_ids,
                                &out_packet,
                            )
                            .await;
                        }

                        Ok(WsInPacket::Broadcast { payload }) => {
                            let out_packet = WsPacket::Message {
                                sender_id: client_uuid,
                                payload,
                            };

                            Box::pin(packet::broadcast_to_room(
                                &state.channel_manager,
                                room_uuid,
                                &out_packet,
                                Some(client_uuid),
                            ))
                            .await;
                        }

                        Err(e) => {
                            error!("Error while parsing client packet {}: {:?}", client_uuid, e);
                        }
                    }
                }

                Ok(AggregatedMessage::Text(_text)) => {
                    let _ = session.text("Server does not support text input").await;
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    let _ = session.pong(&msg).await;
                }

                Ok(AggregatedMessage::Close(_reason)) => {
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

/// Registers the room endpoints into the provided Actix [`Scope`].
pub fn endpoints(scope: Scope) -> Scope {
    scope.service(create_room).service(connect_ws)
}
