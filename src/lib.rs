use crate::channel::ChannelManager;

pub mod channel;
pub mod packet;
pub mod service;
pub mod util;

pub struct AppState {
    pub channel_manager: ChannelManager,
    pub max_packets_per_sec: u32,
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
