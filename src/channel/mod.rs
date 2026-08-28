mod channel_manager;

use std::fmt::Formatter;
use actix_ws::Session;
use uuid::Uuid;
use crate::util::time::get_current_time_in_seconds;

pub use channel_manager::ChannelManager;

#[derive(Clone)]
pub struct Client {
    pub id: Uuid,
    pub session: Session,
}

impl Client {
    pub fn new(id: Uuid, session: Session) -> Self {
        Client { id, session }
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("id", &self.id)
            .field("session", &"<actix_ws::Session>")
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct Channel {
    pub id: Uuid,
    pub clients: Vec<Client>,
    pub time_without_clients: u64,
}

impl Channel {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            id: uuid,
            clients: Vec::new(),
            time_without_clients: get_current_time_in_seconds()
        }
    }
}