use crate::channel::ChannelManager;

pub mod channel;
pub mod packet;
pub mod service;
pub mod util;

pub struct AppState {
    pub channel_manager: ChannelManager,
}
