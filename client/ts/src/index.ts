/**
 * Daiana TypeScript/JavaScript Client Library.
 *
 * High-performance, lightweight binary WebSocket client for room-based peer-to-peer relay communication.
 */

export { DaianaClient } from './client';
export { RoomManager } from './room/roomManager';
export { createRoom, deriveWebSocketUrl } from './room/api';
export { decodePacket } from './codec/decoder';
export {
  encodeBroadcast,
  encodeMulticast,
  encodePacket,
  encodeUnicast,
} from './codec/encoder';
export { ClientOpCode, ServerOpCode } from './protocol/opcodes';
export type {
  ClientConnectedPacket,
  ClientDisconnectedPacket,
  MessagePacket,
  ServerInfoPacket,
  WsInPacket,
} from './protocol/in';
export type {
  BroadcastPacket,
  MulticastPacket,
  UnicastPacket,
  WsOutPacket,
} from './protocol/out';
export type {
  DaianaClientOptions,
  DaianaEvents,
  DaianaEventType,
} from './types';
export { bytesToUuid, isValidUuid, uuidToBytes } from './util/uuid';
