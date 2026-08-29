import { ServerOpCode } from '../protocol/opcodes';
import type { WsInPacket } from '../protocol/in';
import { bytesToUuid } from '../util/uuid';

const textDecoder = new TextDecoder('utf-8');

/**
 * Decodes a raw binary buffer received from the Daiana WebSocket into a typed {@link WsInPacket}.
 *
 * @param buffer The received ArrayBuffer or Uint8Array
 * @returns The parsed {@link WsInPacket}
 * @throws Error if the packet format is invalid or truncated
 */
export function decodePacket(buffer: ArrayBuffer | Uint8Array): WsInPacket {
  const bytes = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);

  if (bytes.length < 1) {
    throw new Error('Incomplete data: buffer is empty');
  }

  const opcode = bytes[0]!;

  switch (opcode) {
    case ServerOpCode.ClientConnected: {
      if (bytes.length < 17) {
        throw new Error(`Incomplete ClientConnected packet: expected at least 17 bytes, got ${bytes.length}`);
      }
      return {
        type: 'connected',
        clientId: bytesToUuid(bytes, 1),
      };
    }

    case ServerOpCode.ClientDisconnected: {
      if (bytes.length < 17) {
        throw new Error(`Incomplete ClientDisconnected packet: expected at least 17 bytes, got ${bytes.length}`);
      }
      return {
        type: 'disconnected',
        clientId: bytesToUuid(bytes, 1),
      };
    }

    case ServerOpCode.Message: {
      if (bytes.length < 17) {
        throw new Error(`Incomplete Message packet: expected at least 17 bytes, got ${bytes.length}`);
      }
      const senderId = bytesToUuid(bytes, 1);
      const payload = bytes.subarray(17);
      return {
        type: 'message',
        senderId,
        payload,
      };
    }

    case ServerOpCode.ServerInfo: {
      const message = textDecoder.decode(bytes.subarray(1));
      return {
        type: 'server_info',
        message,
      };
    }

    default:
      throw new Error(`Invalid or unknown server opcode: 0x${opcode.toString(16).padStart(2, '0')}`);
  }
}
