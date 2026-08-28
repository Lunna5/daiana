use bytes::{Buf, BufMut, Bytes, BytesMut};
use uuid::Uuid;
use crate::packet::ParseError;

const IN_OP_UNICAST: u8 = 0x0;
const IN_OP_MULTICAST: u8 = 0x1;
const IN_OP_BROADCAST: u8 = 0x2;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WsInPacket {
    Unicast {
        target_id: Uuid,
        payload: Bytes,
    },
    Multicast {
        target_ids: Vec<Uuid>,
        payload: Bytes,
    },
    Broadcast {
        payload: Bytes,
    },
}

impl WsInPacket {
    pub fn to_bytes(&self) -> Bytes {
        match self {
            WsInPacket::Unicast { target_id, payload } => {
                let mut buf = BytesMut::with_capacity(17 + payload.len());
                buf.put_u8(IN_OP_UNICAST);
                buf.put_slice(target_id.as_bytes());
                buf.put_slice(payload);
                buf.freeze()
            }
            WsInPacket::Multicast { target_ids, payload } => {
                let mut buf = BytesMut::with_capacity(3 + (target_ids.len() * 16) + payload.len());
                buf.put_u8(IN_OP_MULTICAST);
                buf.put_u16(target_ids.len() as u16);
                for id in target_ids {
                    buf.put_slice(id.as_bytes());
                }
                buf.put_slice(payload);
                buf.freeze()
            }
            WsInPacket::Broadcast { payload } => {
                let mut buf = BytesMut::with_capacity(1 + payload.len());
                buf.put_u8(IN_OP_BROADCAST);
                buf.put_slice(payload);
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
            IN_OP_UNICAST => {
                if data.remaining() < 16 { return Err(ParseError::IncompleteData); }

                let mut uuid_bytes = [0u8; 16];
                data.copy_to_slice(&mut uuid_bytes);

                Ok(WsInPacket::Unicast {
                    target_id: Uuid::from_bytes(uuid_bytes),
                    payload: data,
                })
            }

            IN_OP_MULTICAST => {
                if data.remaining() < 2 { return Err(ParseError::IncompleteData); }

                let count = data.get_u16() as usize;

                if data.remaining() < (count * 16) {
                    return Err(ParseError::IncompleteData);
                }

                let mut target_ids = Vec::with_capacity(count);
                for _ in 0..count {
                    let mut uuid_bytes = [0u8; 16];
                    data.copy_to_slice(&mut uuid_bytes);
                    target_ids.push(Uuid::from_bytes(uuid_bytes));
                }

                Ok(WsInPacket::Multicast {
                    target_ids,
                    payload: data,
                })
            }

            IN_OP_BROADCAST => {
                Ok(WsInPacket::Broadcast {
                    payload: data,
                })
            }

            _ => Err(ParseError::InvalidOpCode(opcode)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_unicast_parsing() {
        let target_id = Uuid::new_v4();
        let mut buf = BytesMut::new();
        buf.put_u8(IN_OP_UNICAST);
        buf.put_slice(target_id.as_bytes());
        buf.put_slice(b"secret message");

        let packet = WsInPacket::from_bytes(buf.freeze()).expect("Failed to parse unicast");
        assert_eq!(
            packet,
            WsInPacket::Unicast {
                target_id,
                payload: Bytes::from_static(b"secret message"),
            }
        );
    }

    #[test]
    fn test_multicast_parsing() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let mut buf = BytesMut::new();
        buf.put_u8(IN_OP_MULTICAST);
        buf.put_u16(2); // 2 targets
        buf.put_slice(id1.as_bytes());
        buf.put_slice(id2.as_bytes());
        buf.put_slice(b"group message");

        let packet = WsInPacket::from_bytes(buf.freeze()).expect("Failed to parse multicast");
        assert_eq!(
            packet,
            WsInPacket::Multicast {
                target_ids: vec![id1, id2],
                payload: Bytes::from_static(b"group message"),
            }
        );
    }

    #[test]
    fn test_broadcast_parsing() {
        let mut buf = BytesMut::new();
        buf.put_u8(IN_OP_BROADCAST);
        buf.put_slice(b"broadcast message to everyone");

        let packet = WsInPacket::from_bytes(buf.freeze()).expect("Failed to parse broadcast");
        assert_eq!(
            packet,
            WsInPacket::Broadcast {
                payload: Bytes::from_static(b"broadcast message to everyone"),
            }
        );
    }

    #[test]
    fn test_invalid_opcode() {
        let buf = Bytes::from_static(&[0x99, 1, 2, 3]);
        let err = WsInPacket::from_bytes(buf).unwrap_err();
        assert_eq!(err, ParseError::InvalidOpCode(0x99));
    }

    #[test]
    fn test_empty_payload() {
        let err = WsInPacket::from_bytes(Bytes::new()).unwrap_err();
        assert_eq!(err, ParseError::IncompleteData);
    }

    #[test]
    fn test_unicast_truncated_uuid() {
        let buf = Bytes::from_static(&[IN_OP_UNICAST, 1, 2, 3, 4]);
        let err = WsInPacket::from_bytes(buf).unwrap_err();
        assert_eq!(err, ParseError::IncompleteData);
    }

    #[test]
    fn test_multicast_truncated_count() {
        let buf = Bytes::from_static(&[IN_OP_MULTICAST, 1]);
        let err = WsInPacket::from_bytes(buf).unwrap_err();
        assert_eq!(err, ParseError::IncompleteData);
    }

    #[test]
    fn test_multicast_truncated_targets() {
        let mut buf = BytesMut::new();
        buf.put_u8(IN_OP_MULTICAST);
        buf.put_u16(2); // claims 2 targets (needs 32 bytes)
        buf.put_slice(Uuid::new_v4().as_bytes()); // only provides 1 target (16 bytes)

        let err = WsInPacket::from_bytes(buf.freeze()).unwrap_err();
        assert_eq!(err, ParseError::IncompleteData);
    }

    #[test]
    fn test_to_bytes_roundtrip() {
        let unicast = WsInPacket::Unicast {
            target_id: Uuid::new_v4(),
            payload: Bytes::from_static(b"unicast payload"),
        };
        assert_eq!(WsInPacket::from_bytes(unicast.to_bytes()).unwrap(), unicast);

        let multicast = WsInPacket::Multicast {
            target_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            payload: Bytes::from_static(b"multicast payload"),
        };
        assert_eq!(WsInPacket::from_bytes(multicast.to_bytes()).unwrap(), multicast);

        let broadcast = WsInPacket::Broadcast {
            payload: Bytes::from_static(b"broadcast payload"),
        };
        assert_eq!(WsInPacket::from_bytes(broadcast.to_bytes()).unwrap(), broadcast);
    }
}