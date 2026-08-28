use crate::channel::{Channel, Client};
use crate::util::error::DaianaError;
use crate::util::time::get_current_time_in_seconds;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug)]
pub struct ChannelManager {
    pub channels: Mutex<HashMap<Uuid, Channel>>,
    pub max_clients_on_room: u16,
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: Mutex::new(HashMap::new()),
            max_clients_on_room: std::env::var("MAX_CLIENTS_ON_CHANNEL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
        }
    }

    pub fn create_channel(&self) -> Uuid {
        let id = Uuid::new_v4();
        let channel = Channel::new(id);

        self.channels
            .lock()
            .expect("Unable to lock channel")
            .insert(id, channel);

        id
    }

    pub fn channel_exists(&self, id: Uuid) -> bool {
        self.channels
            .lock()
            .expect("Unable to lock channel")
            .contains_key(&id)
    }

    pub fn mark_channel_as_active(&self, id: Uuid) -> Result<(), DaianaError> {
        let mut channels = self.channels.lock().expect("Unable to lock channel");

        if let Some(channel) = channels.get_mut(&id) {
            channel.time_without_clients = 0;
            Ok(())
        } else {
            Err(DaianaError::InvalidRoomId)
        }
    }

    pub fn mark_channel_as_not_active(&self, id: Uuid) -> Result<(), DaianaError> {
        let mut channels = self.channels.lock().expect("Unable to lock channel");
        if let Some(channel) = channels.get_mut(&id) {
            channel.time_without_clients = get_current_time_in_seconds();
            Ok(())
        } else {
            Err(DaianaError::InvalidRoomId)
        }
    }

    pub fn insert_client(&self, id: Uuid, client: Client) -> Result<(), DaianaError> {
        let mut channels = self.channels.lock().expect("Unable to lock channel");

        if let Some(channel) = channels.get_mut(&id) {
            if channel.clients.len() >= self.max_clients_on_room as usize {
                return Err(DaianaError::MaximumClientsReached);
            }

            channel.clients.push(client);
            Ok(())
        } else {
            Err(DaianaError::InvalidRoomId)
        }
    }

    pub fn get_clients(&self, id: Uuid) -> Result<Vec<Client>, DaianaError> {
        let channels = self.channels.lock().expect("Unable to lock channel");

        if let Some(channel) = channels.get(&id) {
            Ok(channel.clients.clone())
        } else {
            Err(DaianaError::InvalidRoomId)
        }
    }

    pub fn remove_client(&self, channel_id: Uuid, client_id: Uuid) -> Result<(), DaianaError> {
        let mut channels = self.channels.lock().expect("Unable to lock channel");

        if let Some(channel) = channels.get_mut(&channel_id) {
            channel.clients.retain(|client| client.id != client_id);
            Ok(())
        } else {
            Err(DaianaError::InvalidRoomId)
        }
    }

    pub fn get_client(
        &self,
        channel_id: Uuid,
        client_id: Uuid,
    ) -> Result<Option<Client>, DaianaError> {
        let channels = self.channels.lock().expect("Unable to lock channel");

        if let Some(channel) = channels.get(&channel_id) {
            let client = channel.clients.iter().find(|c| c.id == client_id).cloned();

            Ok(client)
        } else {
            Err(DaianaError::InvalidRoomId)
        }
    }

    pub fn client_exists(&self, channel_id: Uuid, client_id: Uuid) -> Result<bool, DaianaError> {
        let channels = self.channels.lock().expect("Unable to lock channel");

        if let Some(channel) = channels.get(&channel_id) {
            let exists = channel.clients.iter().any(|c| c.id == client_id);
            Ok(exists)
        } else {
            Err(DaianaError::InvalidRoomId)
        }
    }

    pub fn clear_clients(&self, channel_id: Uuid) -> Result<(), DaianaError> {
        let mut channels = self.channels.lock().expect("Unable to lock channel");

        if let Some(channel) = channels.get_mut(&channel_id) {
            channel.clients.clear();
            Ok(())
        } else {
            Err(DaianaError::InvalidRoomId)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_check_channel() {
        let manager = ChannelManager::new();
        let room_id = manager.create_channel();

        assert!(manager.channel_exists(room_id));
        assert!(!manager.channel_exists(Uuid::new_v4()));
    }

    #[test]
    fn test_channel_active_status() {
        let manager = ChannelManager::new();
        let room_id = manager.create_channel();

        assert!(manager.mark_channel_as_active(room_id).is_ok());
        assert!(manager.mark_channel_as_not_active(room_id).is_ok());

        let fake_id = Uuid::new_v4();
        assert!(manager.mark_channel_as_active(fake_id).is_err());
        assert!(manager.mark_channel_as_not_active(fake_id).is_err());
    }

    #[test]
    fn test_operations_on_nonexistent_room() {
        let manager = ChannelManager::new();
        let fake_id = Uuid::new_v4();
        let client_id = Uuid::new_v4();

        assert!(matches!(
            manager.get_clients(fake_id),
            Err(DaianaError::InvalidRoomId)
        ));
        assert!(matches!(
            manager.get_client(fake_id, client_id),
            Err(DaianaError::InvalidRoomId)
        ));
        assert!(matches!(
            manager.client_exists(fake_id, client_id),
            Err(DaianaError::InvalidRoomId)
        ));
        assert!(matches!(
            manager.remove_client(fake_id, client_id),
            Err(DaianaError::InvalidRoomId)
        ));
        assert!(matches!(
            manager.clear_clients(fake_id),
            Err(DaianaError::InvalidRoomId)
        ));
    }
}
