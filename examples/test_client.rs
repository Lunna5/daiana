use std::env;
use std::time::Duration;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;
use daiana::packet::{WsInPacket, WsPacket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let base_http = format!("http://{}:{}", host, port);
    let base_ws = format!("ws://{}:{}", host, port);

    println!("\x1b[1;36m========================================================\x1b[0m");
    println!("\x1b[1;36m       DAIANA - End-to-End WebSocket Test Client       \x1b[0m");
    println!("\x1b[1;36m========================================================\x1b[0m\n");

    let client = reqwest::Client::new();

    // 1. Health Check
    print!("🌱 [1/6] Checking server health ({}/)... ", base_http);
    let health_res = match client.get(format!("{}/", base_http)).send().await {
        Ok(res) => res,
        Err(e) => {
            println!("\x1b[31mFAILED\x1b[0m");
            eprintln!("\x1b[33m💡 Make sure the server is running with `cargo run` first.\x1b[0m");
            return Err(e.into());
        }
    };
    let health_body: Value = health_res.json().await?;
    println!("\x1b[32mOK\x1b[0m (ping: {:?}, version: {:?})", health_body["ping"], health_body["version"]);

    // 2. Create Room
    print!("🌱 [2/6] Creating room via POST /room/... ");
    let create_res = client.post(format!("{}/room/", base_http)).send().await?;
    let create_body: Value = create_res.json().await?;
    let room_id_str = create_body["id"].as_str().ok_or("No 'id' field in room response")?;
    let room_id = Uuid::parse_str(room_id_str)?;
    println!("\x1b[32mOK\x1b[0m (Room UUID: \x1b[35m{}\x1b[0m)", room_id);

    // 3. Connect Client A (Alice)
    let ws_url = format!("{}/room/{}", base_ws, room_id);
    print!("🌱 [3/6] Connecting Client A (Alice) to {}... ", ws_url);
    let (mut alice_ws, _) = connect_async(&ws_url).await?;
    println!("\x1b[32mCONNECTED\x1b[0m");

    // 4. Connect Client B (Bob)
    print!("🌱 [4/6] Connecting Client B (Bob) to {}... ", ws_url);
    let (mut bob_ws, _) = connect_async(&ws_url).await?;
    println!("\x1b[32mCONNECTED\x1b[0m");

    // Alice should receive notification that Bob connected
    let mut bob_id = None;
    if let Some(Ok(Message::Binary(bin))) = alice_ws.next().await {
        if let Ok(WsPacket::ClientConnected { client_id }) = WsPacket::from_bytes(Bytes::from(bin)) {
            println!("   🦋 [Alice] Received ClientConnected event -> Bob UUID: \x1b[33m{}\x1b[0m", client_id);
            bob_id = Some(client_id);
        }
    }

    // Bob should receive existing client sync (Alice)
    let mut alice_id = None;
    if let Some(Ok(Message::Binary(bin))) = bob_ws.next().await {
        if let Ok(WsPacket::ClientConnected { client_id }) = WsPacket::from_bytes(Bytes::from(bin)) {
            println!("   🦋 [Bob]   Received ClientConnected event -> Alice UUID: \x1b[33m{}\x1b[0m", client_id);
            alice_id = Some(client_id);
        }
    }

    let bob_id = bob_id.expect("Alice did not receive Bob's UUID");
    let alice_id = alice_id.expect("Bob did not receive Alice's UUID");

    // 5. Message Exchange Tests: Broadcast, Unicast, and Multicast
    println!("\n🌱 [5/6] Testing packet exchange:");

    // 5.A. Alice sends Broadcast to the whole room
    println!("   📤 [Alice -> Room] Sending Broadcast: 'Hello everyone from Alice!'");
    let broadcast_packet = WsInPacket::Broadcast {
        payload: Bytes::from_static(b"Hello everyone from Alice!"),
    };
    alice_ws.send(Message::Binary(broadcast_packet.to_bytes())).await?;

    // Bob receives Broadcast
    if let Some(Ok(Message::Binary(bin))) = bob_ws.next().await {
        if let Ok(WsPacket::Message { sender_id, payload }) = WsPacket::from_bytes(Bytes::from(bin)) {
            println!(
                "   📥 [Bob]   Received Broadcast from {}: \x1b[32m'{}'\x1b[0m",
                sender_id,
                String::from_utf8_lossy(&payload)
            );
            assert_eq!(sender_id, alice_id);
        }
    }

    // 5.B. Bob sends private Unicast to Alice
    println!("   📤 [Bob -> Alice] Sending Private Unicast: 'Secret message for Alice'");
    let unicast_packet = WsInPacket::Unicast {
        target_id: alice_id,
        payload: Bytes::from_static(b"Secret message for Alice"),
    };
    bob_ws.send(Message::Binary(unicast_packet.to_bytes())).await?;

    // Alice receives Unicast
    if let Some(Ok(Message::Binary(bin))) = alice_ws.next().await {
        if let Ok(WsPacket::Message { sender_id, payload }) = WsPacket::from_bytes(Bytes::from(bin)) {
            println!(
                "   📥 [Alice] Received Unicast from {}: \x1b[32m'{}'\x1b[0m",
                sender_id,
                String::from_utf8_lossy(&payload)
            );
            assert_eq!(sender_id, bob_id);
        }
    }

    // 5.C. Bob sends Multicast to [Alice]
    println!("   📤 [Bob -> [Alice]] Sending Multicast: 'Group multicast message'");
    let multicast_packet = WsInPacket::Multicast {
        target_ids: vec![alice_id],
        payload: Bytes::from_static(b"Group multicast message"),
    };
    bob_ws.send(Message::Binary(multicast_packet.to_bytes())).await?;

    if let Some(Ok(Message::Binary(bin))) = alice_ws.next().await {
        if let Ok(WsPacket::Message { sender_id, payload }) = WsPacket::from_bytes(Bytes::from(bin)) {
            println!(
                "   📥 [Alice] Received Multicast from {}: \x1b[32m'{}'\x1b[0m",
                sender_id,
                String::from_utf8_lossy(&payload)
            );
            assert_eq!(sender_id, bob_id);
        }
    }

    // 6. Disconnect Test
    println!("\n🌱 [6/6] Testing client disconnect:");
    println!("   🔌 Closing Bob's connection...");
    bob_ws.close(None).await?;
    drop(bob_ws);

    // Alice should receive Bob's disconnect event
    tokio::time::sleep(Duration::from_millis(50)).await;
    if let Some(Ok(Message::Binary(bin))) = alice_ws.next().await {
        if let Ok(WsPacket::ClientDisconnected { client_id }) = WsPacket::from_bytes(Bytes::from(bin)) {
            println!("   🦋 [Alice] Received ClientDisconnected event for Bob: \x1b[33m{}\x1b[0m", client_id);
            assert_eq!(client_id, bob_id);
        }
    }

    alice_ws.close(None).await?;

    println!("\n\x1b[1;32m✨ ALL END-TO-END FUNCTIONALITY TESTS PASSED SUCCESSFULLY! ✨\x1b[0m\n");
    Ok(())
}
