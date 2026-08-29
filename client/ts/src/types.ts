/**
 * Configuration options for {@link DaianaClient}.
 */
export interface DaianaClientOptions {
  /**
   * Heartbeat ping interval in milliseconds to keep the WebSocket connection alive through proxies.
   * Default: 25,000 ms (25s). Set to 0 to disable.
   */
  readonly heartbeatInterval?: number;

  /**
   * Connection establishment timeout in milliseconds.
   * Default: 10,000 ms (10s).
   */
  readonly connectionTimeout?: number;

  /**
   * Whether to automatically attempt reconnection when the WebSocket closes unexpectedly.
   * Default: false.
   */
  readonly autoReconnect?: boolean;

  /**
   * Initial delay in milliseconds before attempting to reconnect.
   * Default: 1,000 ms.
   */
  readonly reconnectDelay?: number;

  /**
   * Maximum number of reconnect attempts before giving up.
   * Default: 5.
   */
  readonly maxReconnectAttempts?: number;
}

/**
 * Event map for {@link DaianaClient} listeners.
 */
export interface DaianaEvents {
  /** Dispatched when the WebSocket connects and completes handshake. */
  connected: () => void;

  /** Dispatched when the WebSocket connection is closed. */
  disconnected: (event?: CloseEvent) => void;

  /** Dispatched when another peer joins the room. */
  peer_connected: (clientId: string) => void;

  /** Dispatched when a peer leaves the room. */
  peer_disconnected: (clientId: string) => void;

  /** Dispatched when a message is received from a peer in the room. */
  message: (senderId: string, payload: Uint8Array) => void;

  /** Dispatched when a system or admin notice is received from the server. */
  server_info: (message: string) => void;

  /** Dispatched when a WebSocket error or decode error occurs. */
  error: (error: Error) => void;
}

export type DaianaEventType = keyof DaianaEvents;
