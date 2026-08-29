import { describe, expect, it } from 'vitest';
import { decodePacket } from '../src/codec/decoder';
import {
  encodeBroadcast,
  encodeMulticast,
  encodePacket,
  encodeUnicast,
} from '../src/codec/encoder';
import { ClientOpCode, ServerOpCode } from '../src/protocol/opcodes';
import { uuidToBytes } from '../src/util/uuid';

describe('Codec Encoder & Decoder', () => {
  const encoder = new TextEncoder();
  const sampleUuid1 = 'c1f7b889-4e78-4389-9407-73d8b28cf998';
  const sampleUuid2 = '550e8400-e29b-41d4-a716-446655440000';

  describe('Outbound Encoding', () => {
    it('should encode Unicast packet correctly', () => {
      const payload = encoder.encode('Hello private');
      const bytes = encodeUnicast(sampleUuid1, payload);

      expect(bytes[0]).toBe(ClientOpCode.Unicast);
      expect(bytes.length).toBe(1 + 16 + payload.length);
    });

    it('should encode Multicast packet correctly', () => {
      const payload = encoder.encode('Hello group');
      const targetIds = [sampleUuid1, sampleUuid2];
      const bytes = encodeMulticast(targetIds, payload);

      expect(bytes[0]).toBe(ClientOpCode.Multicast);
      // 1B opcode + 2B count + 2*16B UUIDs + payload
      expect(bytes.length).toBe(1 + 2 + 32 + payload.length);
      // Verify count in big endian
      const count = (bytes[1]! << 8) | bytes[2]!;
      expect(count).toBe(2);
    });

    it('should encode Broadcast packet correctly', () => {
      const payload = encoder.encode('Hello room');
      const bytes = encodeBroadcast(payload);

      expect(bytes[0]).toBe(ClientOpCode.Broadcast);
      expect(bytes.length).toBe(1 + payload.length);
    });

    it('should encode generic WsOutPacket with encodePacket', () => {
      const payload = encoder.encode('test');
      const bytes = encodePacket({ type: 'broadcast', payload });
      expect(bytes[0]).toBe(ClientOpCode.Broadcast);
    });
  });

  describe('Inbound Decoding', () => {
    it('should decode ClientConnected packet', () => {
      const buffer = new Uint8Array(17);
      buffer[0] = ServerOpCode.ClientConnected;
      buffer.set(uuidToBytes(sampleUuid1), 1);

      const packet = decodePacket(buffer);
      expect(packet).toEqual({
        type: 'connected',
        clientId: sampleUuid1,
      });
    });

    it('should decode ClientDisconnected packet', () => {
      const buffer = new Uint8Array(17);
      buffer[0] = ServerOpCode.ClientDisconnected;
      buffer.set(uuidToBytes(sampleUuid1), 1);

      const packet = decodePacket(buffer);
      expect(packet).toEqual({
        type: 'disconnected',
        clientId: sampleUuid1,
      });
    });

    it('should decode Message packet', () => {
      const messageText = 'Peer message';
      const payload = encoder.encode(messageText);
      const buffer = new Uint8Array(17 + payload.length);
      buffer[0] = ServerOpCode.Message;
      buffer.set(uuidToBytes(sampleUuid1), 1);
      buffer.set(payload, 17);

      const packet = decodePacket(buffer);
      expect(packet.type).toBe('message');
      if (packet.type === 'message') {
        expect(packet.senderId).toBe(sampleUuid1);
        expect(new TextDecoder().decode(packet.payload)).toBe(messageText);
      }
    });

    it('should decode ServerInfo packet', () => {
      const infoText = 'Welcome to Daiana!';
      const textBytes = encoder.encode(infoText);
      const buffer = new Uint8Array(1 + textBytes.length);
      buffer[0] = ServerOpCode.ServerInfo;
      buffer.set(textBytes, 1);

      const packet = decodePacket(buffer);
      expect(packet).toEqual({
        type: 'server_info',
        message: infoText,
      });
    });

    it('should throw on unknown opcode', () => {
      const buffer = new Uint8Array([0x99, 1, 2, 3]);
      expect(() => decodePacket(buffer)).toThrow('Invalid or unknown server opcode');
    });

    it('should throw on empty buffer', () => {
      expect(() => decodePacket(new Uint8Array(0))).toThrow('Incomplete data');
    });
  });
});
