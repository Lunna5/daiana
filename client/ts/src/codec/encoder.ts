import { ClientOpCode } from '../protocol/opcodes';
import type { WsOutPacket } from '../protocol/out';
import { uuidToBytes } from '../util/uuid';

/**
 * Encodes a unicast message packet to a binary Uint8Array.
 * Format: `[0x00][16B Target UUID][Payload]`
 *
 * @param targetId The destination peer UUID
 * @param payload The message payload bytes
 */
export function encodeUnicast(targetId: string, payload: Uint8Array): Uint8Array {
  const targetBytes = uuidToBytes(targetId);
  const result = new Uint8Array(1 + 16 + payload.length);
  result[0] = ClientOpCode.Unicast;
  result.set(targetBytes, 1);
  result.set(payload, 17);
  return result;
}

/**
 * Encodes a multicast message packet to a binary Uint8Array.
 * Format: `[0x01][2B Count (u16)][N * 16B UUIDs][Payload]`
 *
 * @param targetIds The array of destination peer UUIDs
 * @param payload The message payload bytes
 */
export function encodeMulticast(targetIds: readonly string[], payload: Uint8Array): Uint8Array {
  if (targetIds.length > 65535) {
    throw new Error(`Target IDs count (${targetIds.length}) exceeds maximum u16 capacity (65535)`);
  }

  const count = targetIds.length;
  const result = new Uint8Array(1 + 2 + count * 16 + payload.length);
  result[0] = ClientOpCode.Multicast;

  // 2 bytes big-endian for target count
  result[1] = (count >> 8) & 0xff;
  result[2] = count & 0xff;

  let offset = 3;
  for (let i = 0; i < count; i++) {
    const targetBytes = uuidToBytes(targetIds[i]!);
    result.set(targetBytes, offset);
    offset += 16;
  }

  result.set(payload, offset);
  return result;
}

/**
 * Encodes a broadcast message packet to a binary Uint8Array.
 * Format: `[0x02][Payload]`
 *
 * @param payload The message payload bytes
 */
export function encodeBroadcast(payload: Uint8Array): Uint8Array {
  const result = new Uint8Array(1 + payload.length);
  result[0] = ClientOpCode.Broadcast;
  result.set(payload, 1);
  return result;
}

/**
 * Encodes any {@link WsOutPacket} into a binary Uint8Array ready to be sent over WebSocket.
 *
 * @param packet The outbound packet to encode
 */
export function encodePacket(packet: WsOutPacket): Uint8Array {
  switch (packet.type) {
    case 'unicast':
      return encodeUnicast(packet.targetId, packet.payload);
    case 'multicast':
      return encodeMulticast(packet.targetIds, packet.payload);
    case 'broadcast':
      return encodeBroadcast(packet.payload);
  }
}
