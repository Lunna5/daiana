# daiana4j ☕

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Java: 25+](https://img.shields.io/badge/Java-25%2B-orange.svg?logo=openjdk)](https://openjdk.org/)
[![Netty](https://img.shields.io/badge/Netty-4.1%2B-green.svg)](https://netty.io/)

A high-performance, fully asynchronous **Java client** for [Daiana](https://github.com/Lunna5/daiana), a room-based binary WebSocket relay server, powered by **Netty**.

---

## ✨ Features

- ⚡ **Asynchronous & Non-blocking:** Built on top of Netty's event loop architecture for maximum throughput and minimal resource usage.
- ☕ **Modern Java:** Leverages Java 21+ records, sealed interfaces, and pattern matching for clean and type-safe packet handling.
- 🔒 **Secure WebSocket (`wss://`) Support:** Native TLS/SSL with SNI (Server Name Indication) support for secure reverse proxies (Cloudflare, Nginx).
- 💓 **Automatic Heartbeat Keep-Alive:** Automatic `PingWebSocketFrame` keep-alive preventing reverse proxies and routers from dropping idle connections.
- 🎯 **Flexible Routing:** Broadcast to all peers in the room, send private Unicast messages to specific UUIDs, or Multicast to selected peer groups.
- 🛠️ **Built-in Room Creation:** Convenient REST client using Java's native `HttpClient` for creating rooms asynchronously (`POST /room/`).

---

## 📦 Installation

### Gradle (Kotlin DSL)
```kotlin
dependencies {
    implementation("dev.lunna:daiana4j:0.0.3")
}
```

### Gradle (Groovy DSL)
```groovy
dependencies {
    implementation 'dev.lunna:daiana4j:0.0.3'
}
```

### Maven (`pom.xml`)
```xml
<dependency>
    <groupId>dev.lunna</groupId>
    <artifactId>daiana4j</artifactId>
    <version>0.0.3</version>
</dependency>
```

---

## 🚀 Quick Start

### 1. Create a Room & Connect

```java
package com.example;

import dev.lunna.daiana4j.DaianaClient;
import dev.lunna.daiana4j.DaianaClientOptions;
import dev.lunna.daiana4j.listener.DaianaListener;
import dev.lunna.daiana4j.room.ServerRoomConnector;

import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.List;
import java.util.UUID;

public class Main {
    public static void main(String[] args) {
        URI serverUri = URI.create("https://daiana.lunna.dev");

        // 1. Create a new room via HTTP POST /room/
        ServerRoomConnector connector = DaianaClient.createRoom(serverUri).join();
        System.out.println("Created room: " + connector.roomId());

        // 2. Configure client options
        DaianaClientOptions options = DaianaClientOptions.create()
                .setHeartbeatInterval(Duration.ofSeconds(25)) // Keep-alive ping interval
                .setConnectionTimeout(Duration.ofSeconds(10));

        // 3. Build the client with event listeners
        DaianaClient client = DaianaClient.builder()
                .uri(connector.websocket()) // wss://daiana.lunna.dev/room/{roomId}
                .options(options)
                .addListener(new DaianaListener() {
                    @Override
                    public void onConnected() {
                        System.out.println("Connected to room!");
                    }

                    @Override
                    public void onClientConnected(UUID clientId) {
                        System.out.println("Peer joined: " + clientId);
                    }

                    @Override
                    public void onClientDisconnected(UUID clientId) {
                        System.out.println("Peer left: " + clientId);
                    }

                    @Override
                    public void onMessage(UUID senderId, byte[] payload) {
                        String text = new String(payload, StandardCharsets.UTF_8);
                        System.out.println("Received message from " + senderId + ": " + text);
                    }

                    @Override
                    public void onServerInfo(String message) {
                        System.out.println("Server notice: " + message);
                    }

                    @Override
                    public void onDisconnected() {
                        System.out.println("Disconnected from server.");
                    }

                    @Override
                    public void onError(Throwable cause) {
                        System.err.println("Error: " + cause.getMessage());
                    }
                })
                .build();

        // 4. Connect asynchronously
        client.connect().join();

        // 5. Broadcast a message to all peers in the room
        client.broadcast("Hello from Java client!".getBytes(StandardCharsets.UTF_8));
    }
}
```

---

## 💬 Sending Messages

### Broadcast (All peers in the room)
```java
byte[] payload = "Hello room!".getBytes(StandardCharsets.UTF_8);
client.broadcast(payload).join();
```

### Unicast (Direct message to a single peer UUID)
```java
UUID targetClientId = UUID.fromString("550e8400-e29b-41d4-a716-446655440000");
byte[] privateMessage = "Whisper message".getBytes(StandardCharsets.UTF_8);

client.sendUnicast(targetClientId, privateMessage).join();
```

### Multicast (Targeted to a list of peer UUIDs)
```java
List<UUID> teamMembers = List.of(
    UUID.fromString("c1f7b889-4e78-4389-9407-73d8b28cf998"),
    UUID.fromString("550e8400-e29b-41d4-a716-446655440000")
);

byte[] teamData = "Group team coordinates".getBytes(StandardCharsets.UTF_8);
client.sendMulticast(teamMembers, teamData).join();
```

---

## ⚙️ Configuration Options (`DaianaClientOptions`)

```java
DaianaClientOptions options = DaianaClientOptions.create()
    .setHeartbeatInterval(Duration.ofSeconds(20))   // Ping keep-alive interval (default: 25s)
    .setConnectionTimeout(Duration.ofSeconds(10))   // Socket connect timeout (default: 10s)
    .setHandshakeTimeout(Duration.ofSeconds(10))    // WebSocket handshake timeout (default: 10s)
    .setMaxContentLength(1024 * 1024)               // Max HTTP aggregator size in bytes (default: 1MiB)
    .setMaxFramePayloadLength(1024 * 1024)          // Max WebSocket frame size in bytes (default: 1MiB)
    .setSslContext(customSslContext);               // Custom Netty SslContext (optional)
```

| Method | Default | Description |
| :--- | :--- | :--- |
| `setHeartbeatInterval(Duration)` | `25 seconds` | Interval for automatic Ping keep-alives (use `Duration.ZERO` to disable). |
| `setConnectionTimeout(Duration)` | `10 seconds` | TCP connection timeout. |
| `setHandshakeTimeout(Duration)` | `10 seconds` | WebSocket HTTP handshake timeout. |
| `setMaxContentLength(int)` | `1,048,576` (1 MiB) | Maximum aggregated HTTP content length in bytes. |
| `setMaxFramePayloadLength(int)` | `1,048,576` (1 MiB) | Maximum WebSocket frame payload size in bytes. |
| `setSslContext(SslContext)` | `null` (auto) | Custom Netty `SslContext` for custom certificates or trust managers. |

---

## 👥 Room Management

The client includes a built-in `RoomManager` that automatically keeps track of active peers in the room as join/leave events are received:

```java
RoomManager roomManager = client.getRoomManager();

// Get list of all currently connected peers in the room
List<UUID> peers = roomManager.getPeers();

// Check if a specific peer is connected
boolean isConnected = roomManager.hasPeer(targetUuid);

// Get total count of other peers
int count = roomManager.getPeerCount();
```

---

## 🛠️ Shared Netty EventLoopGroup

If you run multiple `DaianaClient` instances within a microservice or game backend, you can share a single `EventLoopGroup` across all clients to conserve threads:

```java
EventLoopGroup sharedGroup = new NioEventLoopGroup(4);

DaianaClient client1 = DaianaClient.builder()
        .uri(room1WsUri)
        .eventLoopGroup(sharedGroup) // Shares the event loop
        .build();

DaianaClient client2 = DaianaClient.builder()
        .uri(room2WsUri)
        .eventLoopGroup(sharedGroup)
        .build();
```

---

## 📄 License

This library is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**. See the [LICENSE](../../LICENSE) file for details.
