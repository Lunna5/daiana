use bytes::{Buf, BufMut, Bytes, BytesMut};
use uuid::Uuid;

const OP_CONNECTED: u8 = 0x0;
const OP_DISCONNECTED: u8 = 0x1;
const OP_MESSAGE: u8 = 0x2;
const OP_SERVER_INFO: u8 = 0x3;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    InvalidOpCode(u8),
    IncompleteData,
    InvalidUtf8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsPacket {
    ClientConnected { client_id: Uuid },
    ClientDisconnected { client_id: Uuid },
    Message { sender_id: Uuid, payload: Bytes },
    ServerInfo { message: String },
}

impl WsPacket {
    pub fn to_bytes(&self) -> Bytes {
        match self {
            WsPacket::ClientConnected { client_id } => {
                let mut buf = BytesMut::with_capacity(17);
                buf.put_u8(OP_CONNECTED);
                buf.put_slice(client_id.as_bytes());
                buf.freeze()
            }
            WsPacket::ClientDisconnected { client_id } => {
                let mut buf = BytesMut::with_capacity(17);
                buf.put_u8(OP_DISCONNECTED);
                buf.put_slice(client_id.as_bytes());
                buf.freeze()
            }
            WsPacket::Message { sender_id, payload } => {
                let mut buf = BytesMut::with_capacity(17 + payload.len());
                buf.put_u8(OP_MESSAGE);
                buf.put_slice(sender_id.as_bytes());
                buf.put_slice(payload);
                buf.freeze()
            }
            WsPacket::ServerInfo { message } => {
                let msg_bytes = message.as_bytes();
                let mut buf = BytesMut::with_capacity(1 + msg_bytes.len());
                buf.put_u8(OP_SERVER_INFO);
                buf.put_slice(msg_bytes);
                buf.freeze()
            }
        }
    }

    pub fn from_bytes(mut data: Bytes) -> Result<Self, ParseError> {
        if !data.has_remaining() {
            return Err(ParseError::IncompleteData);
        }

        let opcode = data.get_u8();

        match opcode {
            OP_CONNECTED => {
                if data.remaining() < 16 {
                    return Err(ParseError::IncompleteData);
                }
                let mut uuid_bytes = [0u8; 16];
                data.copy_to_slice(&mut uuid_bytes);
                Ok(WsPacket::ClientConnected {
                    client_id: Uuid::from_bytes(uuid_bytes),
                })
            }
            OP_DISCONNECTED => {
                if data.remaining() < 16 {
                    return Err(ParseError::IncompleteData);
                }
                let mut uuid_bytes = [0u8; 16];
                data.copy_to_slice(&mut uuid_bytes);
                Ok(WsPacket::ClientDisconnected {
                    client_id: Uuid::from_bytes(uuid_bytes),
                })
            }
            OP_MESSAGE => {
                if data.remaining() < 16 {
                    return Err(ParseError::IncompleteData);
                }
                let mut uuid_bytes = [0u8; 16];
                data.copy_to_slice(&mut uuid_bytes);
                Ok(WsPacket::Message {
                    sender_id: Uuid::from_bytes(uuid_bytes),
                    payload: data,
                })
            }
            OP_SERVER_INFO => {
                let text = String::from_utf8(data.to_vec()).map_err(|_| ParseError::InvalidUtf8)?;
                Ok(WsPacket::ServerInfo { message: text })
            }
            _ => Err(ParseError::InvalidOpCode(opcode)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_connected_roundtrip() {
        let client_id = Uuid::new_v4();
        let packet = WsPacket::ClientConnected { client_id };
        let bytes = packet.to_bytes();
        assert_eq!(bytes.len(), 17);
        assert_eq!(bytes[0], OP_CONNECTED);

        let parsed = WsPacket::from_bytes(bytes).expect("Failed to parse packet");
        assert_eq!(parsed, packet);
    }

    #[test]
    fn test_client_disconnected_roundtrip() {
        let client_id = Uuid::new_v4();
        let packet = WsPacket::ClientDisconnected { client_id };
        let bytes = packet.to_bytes();
        assert_eq!(bytes.len(), 17);
        assert_eq!(bytes[0], OP_DISCONNECTED);

        let parsed = WsPacket::from_bytes(bytes).expect("Failed to parse packet");
        assert_eq!(parsed, packet);
    }

    #[test]
    fn test_message_roundtrip() {
        let sender_id = Uuid::new_v4();
        let payload = Bytes::from_static(b"hello world payload");
        let packet = WsPacket::Message {
            sender_id,
            payload: payload.clone(),
        };
        let bytes = packet.to_bytes();
        assert_eq!(bytes[0], OP_MESSAGE);

        let parsed = WsPacket::from_bytes(bytes).expect("Failed to parse message packet");
        assert_eq!(parsed, packet);
    }

    #[test]
    fn test_server_info_roundtrip() {
        let message = "Welcome to the room!".to_string();
        let packet = WsPacket::ServerInfo {
            message: message.clone(),
        };
        let bytes = packet.to_bytes();
        assert_eq!(bytes[0], OP_SERVER_INFO);

        let parsed = WsPacket::from_bytes(bytes).expect("Failed to parse server info");
        assert_eq!(parsed, packet);
    }

    #[test]
    fn test_invalid_opcode() {
        let data = Bytes::from_static(&[0xFF, 0x01, 0x02]);
        let err = WsPacket::from_bytes(data).unwrap_err();
        assert_eq!(err, ParseError::InvalidOpCode(0xFF));
    }

    #[test]
    fn test_empty_data() {
        let data = Bytes::new();
        let err = WsPacket::from_bytes(data).unwrap_err();
        assert_eq!(err, ParseError::IncompleteData);
    }

    #[test]
    fn test_truncated_uuid() {
        let data = Bytes::from_static(&[OP_CONNECTED, 1, 2, 3]);
        let err = WsPacket::from_bytes(data).unwrap_err();
        assert_eq!(err, ParseError::IncompleteData);
    }

    #[test]
    fn test_invalid_utf8_server_info() {
        let data = Bytes::from_static(&[OP_SERVER_INFO, 0xFF, 0xFE]);
        let err = WsPacket::from_bytes(data).unwrap_err();
        assert_eq!(err, ParseError::InvalidUtf8);
    }
}
