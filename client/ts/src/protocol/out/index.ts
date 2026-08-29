/**
 * Outbound packets sent by the client to the Daiana server (Client -> Server).
 */

/**
 * Direct private message packet targeted to a single peer UUID.
 */
export interface UnicastPacket {
  readonly type: 'unicast';
  /** The destination peer UUID. */
  readonly targetId: string;
  /** The raw message payload bytes. */
  readonly payload: Uint8Array;
}

/**
 * Targeted message packet sent to multiple selected peer UUIDs.
 */
export interface MulticastPacket {
  readonly type: 'multicast';
  /** Array of target peer UUIDs. */
  readonly targetIds: readonly string[];
  /** The raw message payload bytes. */
  readonly payload: Uint8Array;
}

/**
 * Broadcast packet sent to all other peers in the room.
 */
export interface BroadcastPacket {
  readonly type: 'broadcast';
  /** The raw message payload bytes. */
  readonly payload: Uint8Array;
}

/**
 * Discriminated union of all outbound packets sent to the Daiana server.
 */
export type WsOutPacket =
  | UnicastPacket
  | MulticastPacket
  | BroadcastPacket;
