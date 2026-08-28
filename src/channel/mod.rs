//! Channel and client membership management.

mod channel_manager;

use crate::util::time::get_current_time_in_seconds;
use actix_ws::Session;
use std::fmt::Formatter;
use uuid::Uuid;

pub use channel_manager::ChannelManager;

/// Represents a connected WebSocket client session in a room.
#[derive(Clone)]
pub struct Client {
    /// The unique identifier assigned to this client connection.
    pub id: Uuid,
    /// The Actix-WS session used to send outbound WebSocket frames.
    pub session: Session,
}

impl Client {
    /// Creates a new [`Client`] with the specified UUID and WebSocket session.
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

/// Represents a room (channel) containing connected clients and tracking inactivity.
#[derive(Clone, Debug)]
pub struct Channel {
    /// The unique room identifier.
    pub id: Uuid,
    /// The list of active clients currently in the room.
    pub clients: Vec<Client>,
    /// Timestamp (UNIX epoch in seconds) when the room became empty (0 if clients are present).
    pub time_without_clients: u64,
}

impl Channel {
    /// Creates a new empty [`Channel`] initialized with the given UUID.
    pub fn new(uuid: Uuid) -> Self {
        Self {
            id: uuid,
            clients: Vec::new(),
            time_without_clients: get_current_time_in_seconds(),
        }
    }
}
