use std::sync::{Arc, Mutex};
use crate::channel::ChannelManager;

pub mod util;
pub mod service;
pub mod channel;
pub mod packet;

pub struct  AppState {
    pub channel_manager: Arc<Mutex<ChannelManager>>,
}