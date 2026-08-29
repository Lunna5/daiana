package dev.lunna.daiana4j.connection;

import dev.lunna.daiana4j.DaianaClientOptions;
import dev.lunna.daiana4j.codec.PacketDecoder;
import dev.lunna.daiana4j.codec.PacketEncoder;
import dev.lunna.daiana4j.listener.DaianaListener;
import io.netty.channel.ChannelInitializer;
import io.netty.channel.ChannelPipeline;
import io.netty.channel.socket.SocketChannel;
import io.netty.handler.codec.http.HttpClientCodec;
import io.netty.handler.codec.http.HttpObjectAggregator;
import io.netty.handler.codec.http.websocketx.WebSocketClientProtocolConfig;
import io.netty.handler.codec.http.websocketx.WebSocketClientProtocolHandler;
import io.netty.handler.codec.http.websocketx.WebSocketVersion;
import io.netty.handler.ssl.SslContext;
import io.netty.handler.ssl.SslContextBuilder;
import org.jetbrains.annotations.NotNull;

import javax.net.ssl.SSLException;
import java.net.URI;
import java.util.List;
import java.util.concurrent.CompletableFuture;

import static java.util.Objects.requireNonNull;

/**
 * Netty {@link ChannelInitializer} configuring the HTTP, TLS/SSL, and WebSocket pipeline for Daiana communication.
 */
public final class DaianaChannelInitializer extends ChannelInitializer<SocketChannel> {
    private final URI uri;
    private final List<DaianaListener> listeners;
    private final CompletableFuture<Void> handshakeFuture;
    private final DaianaClientOptions options;

    /**
     * Constructs a new {@link DaianaChannelInitializer}.
     *
     * @param uri             the target WebSocket URI
     * @param listeners       the list of listeners to receive event notifications
     * @param handshakeFuture the future to complete when the WebSocket handshake succeeds
     * @param options         the client configuration options
     */
    public DaianaChannelInitializer(@NotNull final URI uri,
                                    @NotNull final List<DaianaListener> listeners,
                                    @NotNull final CompletableFuture<Void> handshakeFuture,
                                    @NotNull final DaianaClientOptions options
    ) {
        requireNonNull(uri, "uri cannot be null");
        requireNonNull(listeners, "listeners cannot be null");
        requireNonNull(handshakeFuture, "handshakeFuture cannot be null");
        requireNonNull(options, "options cannot be null");

        this.uri = uri;
        this.listeners = listeners;
        this.handshakeFuture = handshakeFuture;
        this.options = options;
    }

    @Override
    protected void initChannel(SocketChannel ch) throws SSLException {
        ChannelPipeline pipeline = ch.pipeline();

        // 1. SSL/TLS Handler for secure WebSocket connections (wss:// or https://)
        String scheme = uri.getScheme() == null ? "ws" : uri.getScheme();
        boolean isSsl = "wss".equalsIgnoreCase(scheme) || "https".equalsIgnoreCase(scheme);

        if (isSsl) {
            SslContext sslContext = options.getSslContext();
            if (sslContext == null) {
                sslContext = SslContextBuilder.forClient().build();
            }
            String host = uri.getHost();
            int port = uri.getPort() == -1 ? 443 : uri.getPort();
            pipeline.addLast("ssl", sslContext.newHandler(ch.alloc(), host, port));
        }

        // 2. Base HTTP codecs
        pipeline.addLast("http-codec", new HttpClientCodec());
        pipeline.addLast("http-aggregator", new HttpObjectAggregator(options.getMaxContentLength()));

        // 3. Netty WebSocket client protocol handler
        WebSocketClientProtocolConfig wsConfig = WebSocketClientProtocolConfig.newBuilder()
                .webSocketUri(uri)
                .version(WebSocketVersion.V13)
                .subprotocol(null)
                .allowExtensions(true)
                .maxFramePayloadLength(options.getMaxFramePayloadLength())
                .handshakeTimeoutMillis(options.getHandshakeTimeout().toMillis())
                .build();

        pipeline.addLast("ws-protocol", new WebSocketClientProtocolHandler(wsConfig));

        // 4. Daiana binary packet codecs
        pipeline.addLast("packet-encoder", new PacketEncoder());
        pipeline.addLast("packet-decoder", new PacketDecoder());

        // 5. Daiana event handler
        pipeline.addLast("daiana-handler", new DianaClientHandler(listeners, handshakeFuture));
    }
}
