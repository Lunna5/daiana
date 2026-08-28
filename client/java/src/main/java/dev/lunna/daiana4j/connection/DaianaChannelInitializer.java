package dev.lunna.daiana4j.connection;

import dev.lunna.daiana4j.DaianaClientOptions;
import dev.lunna.daiana4j.codec.PacketDecoder;
import dev.lunna.daiana4j.codec.PacketEncoder;
import dev.lunna.daiana4j.listener.DaianaListener;
import io.netty.channel.ChannelInitializer;
import io.netty.channel.ChannelPipeline;
import io.netty.channel.socket.SocketChannel;
import io.netty.handler.codec.http.HttpClientCodec;
import io.netty.handler.codec.http.websocketx.WebSocketClientProtocolConfig;
import io.netty.handler.codec.http.websocketx.WebSocketClientProtocolHandler;
import io.netty.handler.codec.http.websocketx.WebSocketVersion;
import org.jetbrains.annotations.NotNull;

import java.net.URI;
import java.util.List;
import java.util.concurrent.CompletableFuture;

import static java.util.Objects.requireNonNull;

public final class DaianaChannelInitializer extends ChannelInitializer<SocketChannel> {
    private final URI uri;
    private final List<DaianaListener> listeners;
    private final CompletableFuture<Void> handshakeFuture;
    private final DaianaClientOptions options;

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
    protected void initChannel(SocketChannel ch) throws Exception {
        ChannelPipeline pipeline = ch.pipeline();

        pipeline.addLast("http-codec", new HttpClientCodec());
        pipeline.addLast("http-aggregator", new io.netty.handler.codec.http.HttpObjectAggregator(options.getMaxContentLength()));

        WebSocketClientProtocolConfig wsConfig = WebSocketClientProtocolConfig.newBuilder()
                .webSocketUri(uri)
                .version(WebSocketVersion.V13)
                .subprotocol(null)
                .allowExtensions(true)
                .maxFramePayloadLength(options.getMaxFramePayloadLength())
                .build();

        pipeline.addLast("ws-protocol", new WebSocketClientProtocolHandler(wsConfig));

        pipeline.addLast("packet-encoder", new PacketEncoder());
        pipeline.addLast("packet-decoder", new PacketDecoder());

        pipeline.addLast("daiana-handler", new DianaClientHandler(listeners, handshakeFuture));
    }
}
