package dev.lunna.daiana4j;

import dev.lunna.daiana4j.connection.DaianaChannelInitializer;
import dev.lunna.daiana4j.listener.DaianaListener;
import dev.lunna.daiana4j.listener.DefaultDaianaListener;
import dev.lunna.daiana4j.protocol.out.BroadcastPacket;
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

    public static CompletableFuture<ServerRoomConnector> createRoom(URI httpBaseUri) {
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

    public boolean isConnected() {
        return channel != null && channel.isActive();
    }

    public CompletableFuture<Void> broadcast(byte[] payload) {
        return sendPacket(new BroadcastPacket(payload));
    }

    public CompletableFuture<Void> sendUnicast(@NotNull final UUID clientId, byte[] payload) {
        return sendPacket(new UnicastPacket(clientId, payload));
    }

    public CompletableFuture<Void> sendMulticast(@NotNull final List<UUID> clientIds, byte[] payload) {
        return sendPacket(new dev.lunna.daiana4j.protocol.out.MulticastPacket(clientIds, payload));
    }

    public CompletableFuture<Void> sendPacket(@NotNull final WsOutPacket packet) {
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

    @Override
    public void close() throws Exception {
        disconnect().join();
    }
}