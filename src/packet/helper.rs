use futures_util::future::join_all;
use crate::channel::ChannelManager;
use uuid::Uuid;
use log::{error, warn};
use crate::packet::out::WsPacket;

pub async fn disconnect_and_broadcast(
    channel_manager: &ChannelManager,
    room_id: Uuid,
    client_id: Uuid,
) {
    let _ = channel_manager.remove_client(room_id, client_id);

    let disconnect_packet = WsPacket::ClientDisconnected { client_id };

    Box::pin(broadcast_to_room(channel_manager, room_id, &disconnect_packet, None)).await;
}

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

    let mut send_tasks = Vec::new();

    for mut client in clients {
        if Some(client.id) == exclude_client {
            continue;
        }

        let bytes = bytes_to_send.clone();

        let task = async move {
            if let Err(e) = client.session.binary(bytes).await {
                warn!("Failed to send to {}: {}", client.id, e);
                return Some(client.id);
            }
            None
        };

        send_tasks.push(task);
    }

    let failed_clients = join_all(send_tasks).await;

    for failed_id in failed_clients.into_iter().flatten() {
        Box::pin(disconnect_and_broadcast(channel_manager, room_id, failed_id)).await;
    }
}

pub async fn send_to_client(
    channel_manager: &ChannelManager,
    room_id: Uuid,
    target_client_id: Uuid,
    packet: &WsPacket,
) {
    let bytes = packet.to_bytes();

    if let Ok(Some(mut client)) = channel_manager.get_client(room_id, target_client_id) {
        if let Err(e) = client.session.binary(bytes).await {
            warn!("Failed to send direct message to {}: {}", target_client_id, e);

            disconnect_and_broadcast(channel_manager, room_id, target_client_id).await;
        }
    }
}

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

    if let Ok(Some(mut client)) = channel_manager.get_client(room_id, client_id) {
        let _ = client.session.close(None).await;
    }

    disconnect_and_broadcast(channel_manager, room_id, client_id).await;
}

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

    let mut send_tasks = Vec::new();

    for mut client in clients {
        if target_ids.contains(&client.id) {
            let bytes = bytes_to_send.clone();

            let task = async move {
                if let Err(e) = client.session.binary(bytes).await {
                    log::warn!("Failed to multicast to {}: {}", client.id, e);
                    return Some(client.id);
                }
                None
            };

            send_tasks.push(task);
        }
    }

    let failed_clients = join_all(send_tasks).await;

    for failed_id in failed_clients.into_iter().flatten() {
        Box::pin(disconnect_and_broadcast(channel_manager, room_id, failed_id)).await;
    }
}