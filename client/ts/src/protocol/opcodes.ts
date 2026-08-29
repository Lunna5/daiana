/**
 * Protocol opcodes for Daiana binary WebSocket communication.
 */

/**
 * Server-to-Client (Inbound from server) packet opcodes.
 */
export const ServerOpCode = {
  /** A new peer connected to the room. `[0x00][16B UUID]` */
  ClientConnected: 0x00,
  /** A peer disconnected from the room. `[0x01][16B UUID]` */
  ClientDisconnected: 0x01,
  /** A peer sent a message payload. `[0x02][16B Sender UUID][Payload Bytes]` */
  Message: 0x02,
  /** System / admin notice from the server. `[0x03][UTF-8 String]` */
  ServerInfo: 0x03,
} as const;

export type ServerOpCode = (typeof ServerOpCode)[keyof typeof ServerOpCode];

/**
 * Client-to-Server (Outbound to server) packet opcodes.
 */
export const ClientOpCode = {
  /** Direct private message to a specific peer. `[0x00][16B Target UUID][Payload Bytes]` */
  Unicast: 0x00,
  /** Targeted message to multiple peers. `[0x01][2B Count (u16)][N*16B Target UUIDs][Payload Bytes]` */
  Multicast: 0x01,
  /** Broadcast message to all other peers in the room. `[0x02][Payload Bytes]` */
  Broadcast: 0x02,
} as const;

export type ClientOpCode = (typeof ClientOpCode)[keyof typeof ClientOpCode];
