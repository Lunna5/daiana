# daiana4ts 🌸

[![npm version](https://img.shields.io/npm/v/daiana4ts.svg)](https://www.npmjs.com/package/daiana4ts)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.8-blue?logo=typescript)](https://www.typescriptlang.org/)

A lightweight, zero-dependency, high-performance **TypeScript/JavaScript client** for [Daiana](https://github.com/Lunna5/daiana), a room-based binary WebSocket relay server.

---

## ✨ Features

- ⚡ **Zero Runtime Dependencies:** Built strictly on native Web Standards (`WebSocket`, `ArrayBuffer`, `Uint8Array`, `fetch`).
- 🌐 **Isomorphic & Universal:** Runs seamlessly in all modern Browsers, Node.js 18+, Deno, and Bun.
- 🔒 **Secure by Default:** Full support for secure WebSockets (`wss://`) and HTTPS reverse proxies.
- 💓 **Automatic Heartbeat Keep-Alive:** Periodic ping keep-alive preventing silent connection drops on Cloudflare, Nginx, or NAT routers.
- 🎯 **Full Routing Capabilities:** Broadcast to entire room, Unicast to a specific peer UUID, or Multicast to selected UUIDs.
- 🛡️ **Strict TypeScript:** Type-safe events, discriminated unions, and auto-generated declarations.

---

## 📦 Installation

```bash
# Using npm
npm install daiana4ts

# Using pnpm
pnpm add daiana4ts

# Using yarn
yarn add daiana4ts

# Using bun
bun add daiana4ts
```

---

## 🚀 Quick Start

### 1. Create a Room & Connect

```typescript
import { DaianaClient, createRoom, deriveWebSocketUrl } from 'daiana4ts';

async function main() {
  const serverUrl = 'https://daiana.lunna.dev';

  // 1. Create a new room via HTTP POST /room/
  const roomId = await createRoom(serverUrl);
  console.log(`Room created: ${roomId}`);

  // 2. Derive the WebSocket URL (wss://daiana.lunna.dev/room/{roomId})
  const wsUrl = deriveWebSocketUrl(serverUrl, roomId);

  // 3. Initialize the client
  const client = new DaianaClient(wsUrl, {
    heartbeatInterval: 25_000, // Ping every 25s (default)
    autoReconnect: true,
  });

  // 4. Register event listeners
  client.on('connected', () => {
    console.log('Connected to Daiana room!');
  });

  client.on('peer_connected', (peerId) => {
    console.log(`Peer joined the room: ${peerId}`);
  });

  client.on('peer_disconnected', (peerId) => {
    console.log(`Peer left the room: ${peerId}`);
  });

  client.on('message', (senderId, payload) => {
    const text = new TextDecoder().decode(payload);
    console.log(`Message from ${senderId}: ${text}`);
  });

  client.on('server_info', (message) => {
    console.log(`Server notice: ${message}`);
  });

  client.on('error', (err) => {
    console.error('Client error:', err);
  });

  // 5. Establish WebSocket connection
  await client.connect();

  // 6. Send messages (supports string, Uint8Array, or ArrayBuffer)
  client.broadcast('Hello everyone in the room!');
}

main().catch(console.error);
```

---

## 💬 Sending Messages

`DaianaClient` accepts strings, `Uint8Array`, or `ArrayBuffer` payloads:

### Broadcast (All peers in the room)
```typescript
// Broadcast text
client.broadcast('Game starting in 5 seconds!');

// Broadcast binary bytes (e.g. game state, player position)
const binaryData = new Uint8Array([0x01, 0x02, 0x03, 0xFF]);
client.broadcast(binaryData);
```

### Unicast (Direct message to a single peer)
```typescript
client.sendUnicast('550e8400-e29b-41d4-a716-446655440000', 'Private whisper message');
```

### Multicast (Targeted to a list of peers)
```typescript
const teamMembers = [
  'c1f7b889-4e78-4389-9407-73d8b28cf998',
  '550e8400-e29b-41d4-a716-446655440000',
];

client.sendMulticast(teamMembers, 'Team voice data / chat');
```

---

## 📖 API Reference

### `DaianaClient`

```typescript
const client = new DaianaClient(url, options?);
```

#### Options (`DaianaClientOptions`)
| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `heartbeatInterval` | `number` | `25000` | Heartbeat ping interval in ms. Set to `0` to disable. |
| `connectionTimeout` | `number` | `10000` | Connection establishment timeout in ms. |
| `autoReconnect` | `boolean` | `false` | Whether to automatically reconnect when dropped. |
| `reconnectDelay` | `number` | `1000` | Delay in ms before reconnection attempts. |
| `maxReconnectAttempts` | `number` | `5` | Maximum reconnection retry attempts. |

#### Methods
- `connect(): Promise<void>`: Connects to the server and completes the WebSocket handshake.
- `disconnect(): void`: Closes the WebSocket and clears internal state.
- `isConnected(): boolean`: Returns `true` if the connection is currently open.
- `broadcast(payload)`: Broadcasts payload to all other peers in the room.
- `sendUnicast(targetId, payload)`: Sends a private message to a specific peer UUID.
- `sendMulticast(targetIds, payload)`: Sends a message to a list of peer UUIDs.
- `getPeers(): string[]`: Returns an array of UUIDs of all currently connected peers.
- `getRoomManager(): RoomManager`: Returns the internal room peer state manager.
- `on(event, listener)` / `off(event, listener)` / `once(event, listener)`: Event listener subscriptions.

#### Events
| Event | Signature | Description |
| :--- | :--- | :--- |
| `'connected'` | `() => void` | Connection opened and handshake complete. |
| `'disconnected'` | `(event?: CloseEvent) => void` | Connection closed. |
| `'peer_connected'` | `(clientId: string) => void` | Another peer joined the room. |
| `'peer_disconnected'` | `(clientId: string) => void` | A peer left the room. |
| `'message'` | `(senderId: string, payload: Uint8Array) => void` | Message received from a peer. |
| `'server_info'` | `(message: string) => void` | Administrative or server notice. |
| `'error'` | `(error: Error) => void` | Error occurred during connection or decoding. |

---

### Utility Functions

```typescript
import {
  createRoom,
  deriveWebSocketUrl,
  bytesToUuid,
  uuidToBytes,
  isValidUuid,
} from 'daiana4ts';

// Create a room via REST API
const roomId = await createRoom('https://daiana.lunna.dev');

// Derive WebSocket URL
const wsUrl = deriveWebSocketUrl('https://daiana.lunna.dev', roomId);

// Zero-allocation UUID conversions
const bytes = uuidToBytes('550e8400-e29b-41d4-a716-446655440000'); // Uint8Array(16)
const uuidStr = bytesToUuid(bytes); // "550e8400-e29b-41d4-a716-446655440000"
const valid = isValidUuid('invalid-uuid'); // false
```

---

## 📄 License

This library is distributed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**. See the [LICENSE](../../LICENSE) file for details.
