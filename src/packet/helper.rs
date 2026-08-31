//! Helper functions for broadcasting, unicasting, multicasting, and peer synchronization.

use crate::channel::ChannelManager;
use crate::packet::out::WsPacket;
use log::{debug, warn};
use uuid::Uuid;

/// Removes a client from the room and broadcasts a [`WsPacket::ClientDisconnected`] event to remaining peers.
pub async fn disconnect_and_broadcast(
    channel_manager: &ChannelManager,
    room_id: Uuid,
    client_id: Uuid,
) {
    if let Ok(true) = channel_manager.client_exists(room_id, client_id) {
        let _ = channel_manager.remove_client(room_id, client_id);
        let disconnect_packet = WsPacket::ClientDisconnected { client_id };
        Box::pin(broadcast_to_room(
            channel_manager,
            room_id,
            &disconnect_packet,
            None,
        ))
        .await;
    }
}

/// Broadcasts a [`WsPacket::ClientConnected`] event to all existing peers in the room
/// and synchronizes the new client with the list of already connected peers.
pub async fn connect_and_broadcast(
    channel_manager: &ChannelManager,
    room_id: Uuid,
    client_id: Uuid,
) {
    let connect_packet = WsPacket::ClientConnected { client_id };
    Box::pin(broadcast_to_room(
        channel_manager,
        room_id,
        &connect_packet,
        Some(client_id),
    ))
    .await;

    sync_existing_clients(channel_manager, room_id, client_id).await;
}

/// Sends [`WsPacket::ClientConnected`] events to `new_client_id` for each client already in the room.
pub async fn sync_existing_clients(
    channel_manager: &ChannelManager,
    room_id: Uuid,
    new_client_id: Uuid,
) {
    let clients = match channel_manager.get_clients(room_id) {
        Ok(clients) => clients,
        Err(_) => return,
    };

    let mut new_client = match channel_manager.get_client(room_id, new_client_id) {
        Ok(Some(client)) => client,
        _ => return,
    };

    for existing_client in clients {
        if existing_client.id != new_client_id {
            let packet = WsPacket::ClientConnected {
                client_id: existing_client.id,
            };

            if let Err(e) = new_client.session.binary(packet.to_bytes()).await {
                warn!("Failed to sync with new client {}: {}", new_client_id, e);
                Box::pin(disconnect_and_broadcast(
                    channel_manager,
                    room_id,
                    new_client_id,
                ))
                .await;
                break;
            }
        }
    }
}

/// Broadcasts a [`WsPacket`] to all clients in a room, optionally excluding one client.
///
/// Any clients whose connection fails during transmission will be cleanly disconnected and their
/// departure broadcast to the room.
pub async fn broadcast_to_room(
    channel_manager: &ChannelManager,
    room_id: Uuid,
    packet: &WsPacket,
    exclude_client: Option<Uuid>,
) {
    let bytes_to_send = packet.to_bytes();

    let clients = match channel_manager.get_clients(room_id) {
        Ok(clients) => clients,
        Err(_) => return,
    };

    let mut failed_clients = Vec::new();

    for mut client in clients {
        if Some(client.id) == exclude_client {
            continue;
        }

        if let Err(e) = client.session.binary(bytes_to_send.clone()).await {
            debug!("Failed to send to {}: {}", client.id, e);
            failed_clients.push(client.id);
        }
    }

    for failed_id in failed_clients {
        Box::pin(disconnect_and_broadcast(
            channel_manager,
            room_id,
            failed_id,
        ))
        .await;
    }
}

/// Sends a direct private [`WsPacket`] to a specific client in the room.
pub async fn send_to_client(
    channel_manager: &ChannelManager,
    room_id: Uuid,
    target_client_id: Uuid,
    packet: &WsPacket,
) {
    let bytes = packet.to_bytes();

    if let Ok(Some(mut client)) = channel_manager.get_client(room_id, target_client_id)
        && let Err(e) = client.session.binary(bytes).await
    {
        debug!(
            "Failed to send direct message to {}: {}",
            target_client_id, e
        );

        disconnect_and_broadcast(channel_manager, room_id, target_client_id).await;
    }
}

/// Broadcasts a server system info message ([`WsPacket::ServerInfo`]) to all clients in the room.
pub async fn send_server_info_to_room(
    channel_manager: &ChannelManager,
    room_id: Uuid,
    message: &str,
) {
    let packet = WsPacket::ServerInfo {
        message: message.to_string(),
    };

    broadcast_to_room(channel_manager, room_id, &packet, None).await;
}

/// Sends a kick notification to a client, closes their WebSocket session, and disconnects them.
pub async fn kick_client(
    channel_manager: &ChannelManager,
    room_id: Uuid,
    client_id: Uuid,
    reason: &str,
) {
    let info_packet = WsPacket::ServerInfo {
        message: format!("Kicked: {}", reason),
    };
    send_to_client(channel_manager, room_id, client_id, &info_packet).await;

    if let Ok(Some(client)) = channel_manager.get_client(room_id, client_id) {
        let _ = client.session.close(None).await;
    }

    disconnect_and_broadcast(channel_manager, room_id, client_id).await;
}

/// Multicasts a [`WsPacket`] to a specified subset of client UUIDs in the room.
pub async fn multicast_to_clients(
    channel_manager: &ChannelManager,
    room_id: Uuid,
    target_ids: &[Uuid],
    packet: &WsPacket,
) {
    if target_ids.is_empty() {
        return;
    }

    let bytes_to_send = packet.to_bytes();

    let clients = match channel_manager.get_clients(room_id) {
        Ok(clients) => clients,
        Err(_) => return,
    };

    let mut failed_clients = Vec::new();

    for mut client in clients {
        if target_ids.contains(&client.id) {
            if let Err(e) = client.session.binary(bytes_to_send.clone()).await {
                debug!("Failed to multicast to {}: {}", client.id, e);
                failed_clients.push(client.id);
            }
        }
    }

    for failed_id in failed_clients {
        Box::pin(disconnect_and_broadcast(
            channel_manager,
            room_id,
            failed_id,
        ))
        .await;
    }
}
