# Daiana 🍃🪄

A fast, lightweight, and concurrent room-based binary WebSocket relay server built with **Rust** and **Actix Web**.

Daiana provides room management and ultra-low latency real-time communication between multiple clients using a compact binary protocol with built-in support for **Unicast**, **Multicast**, and **Broadcast** messaging.

---

## 🚀 Features

- **Room-Based Architecture:** Create dynamic rooms identified by UUIDs with configurable capacity limits.
- **Ultra-Compact Binary Protocol:** Zero JSON overhead for real-time messaging using efficient byte streams (`bytes::Bytes`).
- **Flexible Routing Modes:**
  - **Broadcast:** Send messages to all clients in the room.
  - **Unicast:** Send private, direct messages to a specific client UUID.
  - **Multicast:** Send messages to a selected list of client UUIDs.
- **Concurrent & Non-Blocking:** Concurrent message delivery powered by `futures_util::future::join_all` and Tokio.
- **Automatic Lifecycle Synchronization:** Clients automatically receive `ClientConnected` and `ClientDisconnected` events with sender UUID verification to prevent identity spoofing.
- **Comprehensive Test Suite:** Includes 24 unit & integration tests plus runnable **Rust** and **TypeScript** end-to-end example clients.

---

## 📦 Binary Protocol Specification

Daiana communicates via raw WebSocket binary frames. The protocol defines two sets of packets: **Client-to-Server (`WsOutPacket`)** and **Server-to-Client (`WsOutPacket`)**.

### 1. Client $\rightarrow$ Server (`WsOutPacket`)

Clients send binary messages starting with an **Opcode** byte followed by the payload structure:

| Opcode | Packet Type | Byte Layout | Description |
| :--- | :--- | :--- | :--- |
| `0x0` | **Unicast** | `[0x0 (1B)] [Target UUID (16B)] [Payload (N Bytes)]` | Sends a private message directly to `target_id`. |
| `0x1` | **Multicast** | `[0x1 (1B)] [Count (2B u16 BigEndian)] [UUID 1 (16B)] ... [Payload (N Bytes)]` | Sends a message to a list of target client UUIDs. |
| `0x2` | **Broadcast** | `[0x2 (1B)] [Payload (N Bytes)]` | Broadcasts the payload to all other clients in the room. |

> **Security Note:** Clients do **not** supply their own `sender_id`. The server strictly assigns and guarantees the authentic `sender_id` (the client's verified session UUID) before forwarding packets.

---

### 2. Server $\rightarrow$ Client (`WsOutPacket`)

The server emits binary frames formatted as follows:

| Opcode | Packet Type | Byte Layout | Description |
| :--- | :--- | :--- | :--- |
| `0x0` | **ClientConnected** | `[0x0 (1B)] [Client UUID (16B)]` | Emitted when a new client joins the room. |
| `0x1` | **ClientDisconnected** | `[0x1 (1B)] [Client UUID (16B)]` | Emitted when a client disconnects (clean or abrupt). |
| `0x2` | **Message** | `[0x2 (1B)] [Sender UUID (16B)] [Payload (N Bytes)]` | Delivered routed message containing the verified sender UUID and payload. |
| `0x3` | **ServerInfo** | `[0x3 (1B)] [UTF-8 String (N Bytes)]` | System / status notifications from the server. |

---

## 🌐 HTTP REST API

| Method | Endpoint | Description | Response Example |
| :--- | :--- | :--- | :--- |
| `GET` | `/` | Health check & version information. | `{"ping": "pong", "version": "0.1.0"}` |
| `GET` | `/stat/` | Active rooms and connected clients statistics. | `{"active_rooms": 1, "active_clients": 2}` |
| `POST` | `/room/` | Creates a new room. | `{"id": "c1f7b889-4e78-4389-9407-73d8b28cf998"}` |
| `GET` | `/room/{id}` | Upgrades the HTTP connection to WebSocket. | WebSocket Handshake (`101 Switching Protocols`) |

---

## ⚙️ Configuration

Environment variables can be defined in a `.env` file at the root of the project:

```env
# Server bind address and port
HOST=0.0.0.0
PORT=8080

# Maximum clients allowed per room (default: 5)
MAX_CLIENTS_ON_CHANNEL=5

# Room inactivity timeout in seconds (default: 30)
CHANNEL_TIMEOUT=30

# Maximum packets per second per client connection (default: 100, 0 to disable)
MAX_PACKETS_PER_SEC=100

# Maximum packet payload size in bytes (default: 65536 / 64KiB)
MAX_PACKET_SIZE_BYTES=65536

# CORS Configuration (Disabled by default if omitted or ENABLE_CORS=false)
# ENABLE_CORS=true
# CORS_ORIGINS=*
# CORS_METHODS=GET,POST,OPTIONS
# CORS_HEADERS=Authorization,Accept,Content-Type
# CORS_MAX_AGE=3600

# Log level filter (trace, debug, info, warn, error)
RUST_LOG=info
```

---

## 🛠️ Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/) (2024 edition or latest stable)
- (Optional) [Node.js](https://nodejs.org/) / `npx` (to run the TypeScript test client)

### 1. Build and Run Server

#### Using Docker (Pre-built Image):
```bash
docker run -d -p 8080:8080 --name daiana ghcr.io/lunna5/daiana:latest
```

#### Using Docker Compose:
Create a `docker-compose.yml` file:

```yaml
services:
  daiana:
    image: ghcr.io/lunna5/daiana:latest
    container_name: daiana
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - HOST=0.0.0.0
      - PORT=8080
      - MAX_CLIENTS_ON_CHANNEL=5
      - CHANNEL_TIMEOUT=30
      - MAX_PACKETS_PER_SEC=100
      - MAX_PACKET_SIZE_BYTES=65536
      - RUST_LOG=info
      # - ENABLE_CORS=true
      # - CORS_ORIGINS=*
```

Run with:
```bash
docker compose up -d
```

#### Using Cargo (Locally):
```bash
# Clone the repository
git clone https://github.com/Lunna5/daiana.git
cd daiana

# Start the server
cargo run
```

#### Building Docker Image Locally:
```bash
docker build -t daiana .
docker run -p 8080:8080 daiana
```

The server will initialize on `http://0.0.0.0:8080`.

---

### 2. Run Tests

Execute the complete unit and integration test suite:

```bash
cargo test
```

---

### 3. Run Example Clients

Daiana includes two end-to-end client scripts demonstrating how to interact with the server:

#### 🦀 Rust Client Example:
```bash
cargo run --example test_client
```

#### 🌐 TypeScript Client Example:
```bash
pnpx tsx examples/node/test_client.ts
```

Both examples automate:
1. Verifying server health (`GET /`).
2. Creating a room (`POST /room/`).
3. Connecting two clients (`Alice` and `Bob`) via WebSocket.
4. Sending **Broadcast**, **Unicast**, and **Multicast** binary packets.
5. Verifying packet reception and disconnecting cleanly.

---

## 📄 License

This project is licensed under the GNU-AGPL License.
