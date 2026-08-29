//! # Daiana 🌸
//!
//! A lightweight, high-performance, real-time room-based binary WebSocket relay server.
//!
//! ## Overview
//! Daiana is designed for low-latency peer-to-peer communication over WebSockets using a zero-overhead
//! binary protocol. Clients connect to isolated rooms identified by UUIDs and can communicate via:
//!
//! - **Broadcast:** Message delivered to all other peers in the room.
//! - **Unicast:** Private message routed directly to a single peer's UUID.
//! - **Multicast:** Targeted message delivered to a selected list of peer UUIDs.

use crate::channel::ChannelManager;

pub mod channel;
pub mod config;
pub mod packet;
pub mod service;
pub mod util;
/// Shared application state managed by Actix-web.
///
/// Contains the channel manager instance and server-wide rate limiting / packet size limits.
pub struct AppState {
    /// The global channel manager maintaining all active rooms and their client sessions.
    pub channel_manager: ChannelManager,

    /// Maximum incoming binary packets per second permitted per client connection (0 to disable).
    pub max_packets_per_sec: u32,

    /// Maximum payload size in bytes permitted per binary packet (0 to disable).
    pub max_packet_size_bytes: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            channel_manager: ChannelManager::new(),
            max_packets_per_sec: 100,
            max_packet_size_bytes: 65_536,
        }
    }
}
