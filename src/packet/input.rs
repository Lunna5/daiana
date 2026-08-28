use bytes::{Buf, Bytes};
use uuid::Uuid;
use crate::packet::ParseError;

const IN_OP_UNICAST: u8 = 0x0;
const IN_OP_MULTICAST: u8 = 0x1;
const IN_OP_BROADCAST: u8 = 0x2;

#[derive(Debug)]
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