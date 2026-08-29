/**
 * Inbound packets received by the client from the Daiana server (Server -> Client).
 */

/**
 * Event notification when a new peer joins the room.
 */
export interface ClientConnectedPacket {
  readonly type: 'connected';
  /** The UUID of the connected peer. */
  readonly clientId: string;
}

/**
 * Event notification when a peer leaves the room.
 */
export interface ClientDisconnectedPacket {
  readonly type: 'disconnected';
  /** The UUID of the disconnected peer. */
  readonly clientId: string;
}

/**
 * Message packet received from another peer in the room.
 */
export interface MessagePacket {
  readonly type: 'message';
  /** The verified UUID of the sender. */
  readonly senderId: string;
  /** The raw message payload bytes. */
  readonly payload: Uint8Array;
}

/**
 * System / administrative message from the Daiana server.
 */
export interface ServerInfoPacket {
  readonly type: 'server_info';
  /** The server status or kick reason message. */
  readonly message: string;
}

/**
 * Discriminated union of all possible inbound packets from the Daiana server.
 */
export type WsInPacket =
  | ClientConnectedPacket
  | ClientDisconnectedPacket
  | MessagePacket
  | ServerInfoPacket;
