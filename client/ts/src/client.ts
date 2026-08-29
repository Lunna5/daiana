import { decodePacket } from './codec/decoder';
import { encodeBroadcast, encodeMulticast, encodeUnicast } from './codec/encoder';
import { createRoom, deriveWebSocketUrl } from './room/api';
import { RoomManager } from './room/roomManager';
import type { DaianaClientOptions, DaianaEvents, DaianaEventType } from './types';

const textEncoder = new TextEncoder();

/**
 * High-performance, asynchronous WebSocket client for connecting to a Daiana room.
 */
export class DaianaClient {
  public static readonly createRoom = createRoom;
  public static readonly deriveWebSocketUrl = deriveWebSocketUrl;

  private readonly url: string;
  private readonly options: Required<DaianaClientOptions>;
  private readonly roomManager: RoomManager = new RoomManager();
  private readonly listeners: {
    [K in DaianaEventType]?: Set<DaianaEvents[K]>;
  } = {};

  private ws: WebSocket | null = null;
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  private isManuallyClosed: boolean = false;
  private reconnectAttempts: number = 0;

  /**
   * Constructs a new {@link DaianaClient} instance.
   *
   * @param url The full WebSocket URL pointing to `/room/{roomId}` (e.g. `wss://daiana.lunna.dev/room/...`)
   * @param options Client configuration options
   */
  constructor(url: string, options: DaianaClientOptions = {}) {
    if (!url) {
      throw new Error('WebSocket URL cannot be empty');
    }

    this.url = url;
    this.options = {
      heartbeatInterval: options.heartbeatInterval ?? 25_000,
      connectionTimeout: options.connectionTimeout ?? 10_000,
      autoReconnect: options.autoReconnect ?? false,
      reconnectDelay: options.reconnectDelay ?? 1_000,
      maxReconnectAttempts: options.maxReconnectAttempts ?? 5,
    };
  }

  /**
   * Registers an event listener.
   *
   * @param event The event name
   * @param listener The callback function
   * @returns `this` instance for chaining
   */
  public on<K extends DaianaEventType>(event: K, listener: DaianaEvents[K]): this {
    if (!this.listeners[event]) {
      this.listeners[event] = new Set() as any;
    }
    this.listeners[event]!.add(listener);
    return this;
  }

  /**
   * Removes an event listener.
   *
   * @param event The event name
   * @param listener The callback function to remove
   * @returns `this` instance for chaining
   */
  public off<K extends DaianaEventType>(event: K, listener: DaianaEvents[K]): this {
    this.listeners[event]?.delete(listener);
    return this;
  }

  /**
   * Registers a one-time event listener.
   *
   * @param event The event name
   * @param listener The callback function
   * @returns `this` instance for chaining
   */
  public once<K extends DaianaEventType>(event: K, listener: DaianaEvents[K]): this {
    const onceWrapper = ((...args: any[]) => {
      this.off(event, onceWrapper as any);
      (listener as any)(...args);
    }) as DaianaEvents[K];

    return this.on(event, onceWrapper);
  }

  /**
   * Connects to the Daiana WebSocket room asynchronously.
   *
   * @returns A Promise resolving when the WebSocket connection and handshake are successfully opened
   */
  public async connect(): Promise<void> {
    this.isManuallyClosed = false;

    return new Promise((resolve, reject) => {
      let isResolved = false;
      const timeoutTimer = setTimeout(() => {
        if (!isResolved) {
          isResolved = true;
          this.ws?.close();
          const err = new Error(`Connection timeout after ${this.options.connectionTimeout}ms`);
          this.emit('error', err);
          reject(err);
        }
      }, this.options.connectionTimeout);

      try {
        this.ws = new WebSocket(this.url);
        this.ws.binaryType = 'arraybuffer';

        this.ws.onopen = () => {
          if (!isResolved) {
            isResolved = true;
            clearTimeout(timeoutTimer);
            this.reconnectAttempts = 0;
            this.startHeartbeat();
            this.emit('connected');
            resolve();
          }
        };

        this.ws.onerror = (evt) => {
          const err = new Error(`WebSocket error event: ${evt.type}`);
          this.emit('error', err);
          if (!isResolved) {
            isResolved = true;
            clearTimeout(timeoutTimer);
            reject(err);
          }
        };

        this.ws.onclose = (event) => {
          this.stopHeartbeat();
          this.roomManager.clear();
          this.emit('disconnected', event);

          if (!isResolved) {
            isResolved = true;
            clearTimeout(timeoutTimer);
            reject(new Error(`WebSocket closed before opening (code: ${event.code})`));
          }

          if (!this.isManuallyClosed && this.options.autoReconnect) {
            this.handleReconnect();
          }
        };

        this.ws.onmessage = (event: MessageEvent) => {
          this.handleIncomingMessage(event.data);
        };
      } catch (err) {
        clearTimeout(timeoutTimer);
        const error = err instanceof Error ? err : new Error(String(err));
        this.emit('error', error);
        reject(error);
      }
    });
  }

  /**
   * Disconnects and closes the WebSocket connection.
   */
  public disconnect(): void {
    this.isManuallyClosed = true;
    this.stopHeartbeat();
    this.roomManager.clear();

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * Checks if the WebSocket connection is currently active and open.
   */
  public isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }

  /**
   * Returns the internal {@link RoomManager} maintaining all active peers.
   */
  public getRoomManager(): RoomManager {
    return this.roomManager;
  }

  /**
   * Returns a snapshot array of all peer UUIDs currently in the room.
   */
  public getPeers(): string[] {
    return this.roomManager.getPeers();
  }

  /**
   * Broadcasts a payload to all other peers in the room.
   *
   * @param payload The raw byte array, ArrayBuffer, or string payload
   */
  public broadcast(payload: Uint8Array | ArrayBuffer | string): void {
    const bytes = this.normalizePayload(payload);
    const packet = encodeBroadcast(bytes);
    this.sendRaw(packet);
  }

  /**
   * Sends a direct private message (unicast) to a specific peer UUID in the room.
   *
   * @param targetId The destination peer's UUID
   * @param payload The raw byte array, ArrayBuffer, or string payload
   */
  public sendUnicast(targetId: string, payload: Uint8Array | ArrayBuffer | string): void {
    const bytes = this.normalizePayload(payload);
    const packet = encodeUnicast(targetId, bytes);
    this.sendRaw(packet);
  }

  /**
   * Sends a targeted message (multicast) to a list of peer UUIDs in the room.
   *
   * @param targetIds Array of destination peer UUIDs
   * @param payload The raw byte array, ArrayBuffer, or string payload
   */
  public sendMulticast(targetIds: readonly string[], payload: Uint8Array | ArrayBuffer | string): void {
    const bytes = this.normalizePayload(payload);
    const packet = encodeMulticast(targetIds, bytes);
    this.sendRaw(packet);
  }

  /**
   * Sends a raw binary packet over the WebSocket.
   */
  private sendRaw(data: Uint8Array): void {
    if (!this.isConnected() || !this.ws) {
      throw new Error('Cannot send packet: WebSocket is not connected');
    }
    this.ws.send(data as unknown as BufferSource);
  }

  private handleIncomingMessage(data: unknown): void {
    try {
      if (data instanceof ArrayBuffer || data instanceof Uint8Array) {
        const packet = decodePacket(data);

        switch (packet.type) {
          case 'connected':
            this.roomManager.addPeer(packet.clientId);
            this.emit('peer_connected', packet.clientId);
            break;

          case 'disconnected':
            this.roomManager.removePeer(packet.clientId);
            this.emit('peer_disconnected', packet.clientId);
            break;

          case 'message':
            this.emit('message', packet.senderId, packet.payload);
            break;

          case 'server_info':
            this.emit('server_info', packet.message);
            break;
        }
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      this.emit('error', error);
    }
  }

  private emit<K extends DaianaEventType>(event: K, ...args: Parameters<DaianaEvents[K]>): void {
    const set = this.listeners[event];
    if (set) {
      for (const listener of set) {
        try {
          (listener as any)(...args);
        } catch (e) {
          console.error(`[DaianaClient] Error in "${event}" listener:`, e);
        }
      }
    }
  }

  private normalizePayload(payload: Uint8Array | ArrayBuffer | string): Uint8Array {
    if (typeof payload === 'string') {
      return textEncoder.encode(payload);
    }
    if (payload instanceof ArrayBuffer) {
      return new Uint8Array(payload);
    }
    return payload;
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();

    if (this.options.heartbeatInterval <= 0) {
      return;
    }

    this.heartbeatTimer = setInterval(() => {
      if (this.isConnected() && this.ws) {
        // If native ws.ping is supported (e.g. Node ws), use it; otherwise WebSocket automatically handles server ping
        if (typeof (this.ws as any).ping === 'function') {
          (this.ws as any).ping();
        }
      }
    }, this.options.heartbeatInterval);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  private handleReconnect(): void {
    if (this.reconnectAttempts >= this.options.maxReconnectAttempts) {
      return;
    }

    this.reconnectAttempts++;
    setTimeout(() => {
      if (!this.isManuallyClosed) {
        this.connect().catch((err) => {
          this.emit('error', new Error(`Reconnection attempt ${this.reconnectAttempts} failed: ${err.message}`));
        });
      }
    }, this.options.reconnectDelay);
  }
}
