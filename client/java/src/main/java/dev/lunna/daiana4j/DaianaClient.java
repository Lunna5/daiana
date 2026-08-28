package dev.lunna.daiana4j;

import dev.lunna.daiana4j.connection.DaianaChannelInitializer;
import dev.lunna.daiana4j.listener.DaianaListener;
import dev.lunna.daiana4j.listener.DefaultDaianaListener;
import dev.lunna.daiana4j.protocol.out.BroadcastPacket;
import dev.lunna.daiana4j.protocol.out.MulticastPacket;
import dev.lunna.daiana4j.protocol.out.UnicastPacket;
import dev.lunna.daiana4j.protocol.out.WsOutPacket;
import dev.lunna.daiana4j.room.RoomManager;
import dev.lunna.daiana4j.room.ServerRoomConnector;
import io.netty.bootstrap.Bootstrap;
import io.netty.channel.Channel;
import io.netty.channel.ChannelFutureListener;
import io.netty.channel.ChannelOption;
import io.netty.channel.EventLoopGroup;
import io.netty.channel.nio.NioEventLoopGroup;
import io.netty.channel.socket.nio.NioSocketChannel;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.util.List;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import static java.util.Objects.requireNonNull;

/**
 * High-performance, asynchronous Java client for the Daiana binary WebSocket relay server.
 * <p>
 * This client manages the underlying Netty event loop, channel pipeline, WebSocket handshake,
 * and packet dispatching. It supports broadcasting messages, sending direct private messages (unicast),
 * and multicasting to selected peers.
 *
 * <p>Example usage:
 * <pre>{@code
 * URI roomUri = URI.create("ws://localhost:8080/room/4b045f44-8d48-436f-b2b0-062e7ea66b7a");
 *
 * DaianaClient client = DaianaClientBuilder.create()
 *         .serverUri(roomUri)
 *         .addListener(new DaianaListener() {
 *             @Override
 *             public void onMessage(UUID senderId, byte[] payload) {
 *                 System.out.println("Received: " + new String(payload));
 *             }
 *         })
 *         .build();
 *
 * client.connect().join();
 * client.broadcast("Hello room!".getBytes(StandardCharsets.UTF_8)).join();
 * }</pre>
 */
public final class DaianaClient implements AutoCloseable {
    private static final HttpClient HTTP_CLIENT = HttpClient.newHttpClient();
    private static final Pattern ROOM_ID_PATTERN = Pattern.compile("\"id\"\\s*:\\s*\"([^\"]+)\"");

    private final URI uri;
    private final List<DaianaListener> listeners;
    private final DaianaClientOptions options;
    private final RoomManager roomManager = new RoomManager();

    private final EventLoopGroup group;
    private final boolean managesGroup;

    private Channel channel;

    /**
     * Constructs a new {@link DaianaClient}.
     *
     * @param uri              the WebSocket URI pointing to the target room (e.g. {@code ws://host:port/room/{roomId}})
     * @param listeners        the list of event listeners to be notified upon packet reception and connection events
     * @param options          the client configuration options
     * @param sharedGroup      optional shared {@link EventLoopGroup}; if {@code null}, a new {@link NioEventLoopGroup} is managed
     * @param defaultListeners whether to register the {@link DefaultDaianaListener} for automatic room peer tracking
     */
    public DaianaClient(@NotNull final URI uri,
                        @NotNull final List<DaianaListener> listeners,
                        @NotNull final DaianaClientOptions options,
                        @Nullable final EventLoopGroup sharedGroup,
                        final boolean defaultListeners
    ) {
        this.uri = requireNonNull(uri, "uri cannot be null");
        this.listeners = requireNonNull(listeners, "listener cannot be null");
        this.options = requireNonNull(options, "options cannot be null");

        if (sharedGroup != null) {
            this.group = sharedGroup;
            this.managesGroup = false;
        } else {
            this.group = new NioEventLoopGroup();
            this.managesGroup = true;
        }

        if (defaultListeners) {
            this.listeners.add(new DefaultDaianaListener(roomManager));
        }
    }

    /**
     * Sends an HTTP {@code POST /room/} request to the Daiana server to create a new room.
     *
     * @param httpBaseUri the base HTTP URI of the server (e.g. {@code http://localhost:8080})
     * @return a {@link CompletableFuture} resolving to a {@link ServerRoomConnector} containing the created room's UUID
     */
    public static CompletableFuture<ServerRoomConnector> createRoom(@NotNull final URI httpBaseUri) {
        requireNonNull(httpBaseUri, "httpBaseUri cannot be null");
        String base = httpBaseUri.toString().replaceAll("/+$", "");
        URI endpoint = URI.create(base + "/room/");

        HttpRequest request = HttpRequest.newBuilder()
                .uri(endpoint)
                .POST(HttpRequest.BodyPublishers.noBody())
                .header("Accept", "application/json")
                .build();

        return HTTP_CLIENT.sendAsync(request, HttpResponse.BodyHandlers.ofString())
                .thenApply(response -> {
                    if (response.statusCode() != 200) {
                        throw new IllegalStateException("Failed to create room. HTTP " +
                                response.statusCode() + ": " + response.body());
                    }

                    Matcher matcher = ROOM_ID_PATTERN.matcher(response.body());
                    if (matcher.find()) {
                        return new ServerRoomConnector(httpBaseUri, UUID.fromString(matcher.group(1)));
                    }
                    throw new IllegalStateException("Invalid JSON response from server: " + response.body());
                });
    }

    /**
     * Connects to the Daiana WebSocket room asynchronously and performs the WebSocket handshake.
     *
     * @return a {@link CompletableFuture} completing when the WebSocket handshake has successfully finished
     */
    public CompletableFuture<Void> connect() {
        CompletableFuture<Void> handshakeFuture = new CompletableFuture<>();

        String host = uri.getHost();
        int port = uri.getPort() == -1 ? (uri.getScheme().equals("wss") ? 443 : 80) : uri.getPort();

        Bootstrap bootstrap = new Bootstrap()
                .group(group)
                .channel(NioSocketChannel.class)
                .option(ChannelOption.CONNECT_TIMEOUT_MILLIS, (int) options.getConnectionTimeout().toMillis())
                .option(ChannelOption.SO_KEEPALIVE, true)
                .option(ChannelOption.TCP_NODELAY, true)
                .handler(new DaianaChannelInitializer(uri, listeners, handshakeFuture, options));

        bootstrap.connect(host, port).addListener((ChannelFutureListener) future -> {
            if (future.isSuccess()) {
                channel = future.channel();
            } else {
                handshakeFuture.completeExceptionally(future.cause());
            }
        });

        return handshakeFuture;
    }

    /**
     * Checks if the WebSocket channel is currently open and active.
     *
     * @return {@code true} if connected and active, {@code false} otherwise
     */
    public boolean isConnected() {
        return channel != null && channel.isActive();
    }

    /**
     * Returns the {@link RoomManager} maintaining the list of active connected peers in this room.
     *
     * @return the {@link RoomManager} instance
     */
    public @NotNull RoomManager getRoomManager() {
        return roomManager;
    }

    /**
     * Broadcasts a binary payload to all other clients connected to the room.
     *
     * @param payload the raw byte array to broadcast
     * @return a {@link CompletableFuture} completing when the packet is written and flushed
     */
    public CompletableFuture<Void> broadcast(@NotNull final byte[] payload) {
        requireNonNull(payload, "payload cannot be null");
        return sendPacket(new BroadcastPacket(payload));
    }

    /**
     * Sends a private direct message (unicast) to a specific client in the room.
     *
     * @param clientId the destination client's {@link UUID}
     * @param payload  the raw byte array to send
     * @return a {@link CompletableFuture} completing when the packet is written and flushed
     */
    public CompletableFuture<Void> sendUnicast(@NotNull final UUID clientId, @NotNull final byte[] payload) {
        requireNonNull(clientId, "clientId cannot be null");
        requireNonNull(payload, "payload cannot be null");
        return sendPacket(new UnicastPacket(clientId, payload));
    }

    /**
     * Sends a message to multiple specified clients (multicast) in the room.
     *
     * @param clientIds the list of destination client {@link UUID}s
     * @param payload   the raw byte array to send
     * @return a {@link CompletableFuture} completing when the packet is written and flushed
     */
    public CompletableFuture<Void> sendMulticast(@NotNull final List<UUID> clientIds, @NotNull final byte[] payload) {
        requireNonNull(clientIds, "clientIds cannot be null");
        requireNonNull(payload, "payload cannot be null");
        return sendPacket(new MulticastPacket(clientIds, payload));
    }

    /**
     * Sends an outbound packet ({@link WsOutPacket}) across the WebSocket channel.
     *
     * @param packet the packet to serialize and send
     * @return a {@link CompletableFuture} completing when the packet is written and flushed
     */
    public CompletableFuture<Void> sendPacket(@NotNull final WsOutPacket packet) {
        requireNonNull(packet, "packet cannot be null");
        if (!isConnected()) {
            return CompletableFuture.failedFuture(new IllegalStateException("Client is not connected"));
        }

        CompletableFuture<Void> writeFuture = new CompletableFuture<>();
        channel.writeAndFlush(packet).addListener(future -> {
            if (future.isSuccess()) {
                writeFuture.complete(null);
            } else {
                writeFuture.completeExceptionally(future.cause());
            }
        });

        return writeFuture;
    }

    /**
     * Disconnects the WebSocket channel and gracefully shuts down the managed {@link EventLoopGroup} if owned.
     *
     * @return a {@link CompletableFuture} completing when the disconnect process has finished
     */
    public CompletableFuture<Void> disconnect() {
        CompletableFuture<Void> disconnectFuture = new CompletableFuture<>();

        if (channel != null && channel.isActive()) {
            channel.close().addListener(f -> {
                shutdownGroupIfOwned();
                disconnectFuture.complete(null);
            });
        } else {
            shutdownGroupIfOwned();
            disconnectFuture.complete(null);
        }

        return disconnectFuture;
    }

    private void shutdownGroupIfOwned() {
        if (managesGroup && !group.isShutdown()) {
            group.shutdownGracefully();
        }
    }

    /**
     * Closes this client synchronously by awaiting {@link #disconnect()}.
     */
    @Override
    public void close() {
        disconnect().join();
    }
}