package dev.lunna.daiana4j.connection;

import dev.lunna.daiana4j.listener.DaianaListener;
import dev.lunna.daiana4j.protocol.in.*;
import io.netty.channel.ChannelHandlerContext;
import io.netty.channel.SimpleChannelInboundHandler;
import io.netty.handler.codec.http.websocketx.WebSocketClientProtocolHandler;
import org.jetbrains.annotations.NotNull;

import java.util.List;
import java.util.concurrent.CompletableFuture;

import static java.util.Objects.requireNonNull;

/**
 * Netty inbound handler that handles the WebSocket handshake completion event,
 * dispatches decoded {@link WsInPacket}s to registered {@link DaianaListener}s,
 * and handles connection lifecycle events.
 */
public final class DianaClientHandler extends SimpleChannelInboundHandler<WsInPacket> {
    private final List<DaianaListener> listeners;
    private final CompletableFuture<Void> handshakeFuture;

    /**
     * Constructs a new {@link DianaClientHandler}.
     *
     * @param listeners       the list of listeners to notify
     * @param handshakeFuture the future to complete on handshake completion
     */
    public DianaClientHandler(@NotNull List<DaianaListener> listeners, @NotNull CompletableFuture<Void> handshakeFuture) {
        this.listeners = requireNonNull(listeners, "listeners cannot be null");
        this.handshakeFuture = requireNonNull(handshakeFuture, "handshakeFuture cannot be null");
    }

    @Override
    public void userEventTriggered(ChannelHandlerContext ctx, Object evt) throws Exception {
        if (evt instanceof WebSocketClientProtocolHandler.ClientHandshakeStateEvent event) {
            if (event == WebSocketClientProtocolHandler.ClientHandshakeStateEvent.HANDSHAKE_COMPLETE) {
                handshakeFuture.complete(null);
                for (DaianaListener listener : listeners) {
                    listener.onConnected();
                }
            }
        }

        super.userEventTriggered(ctx, evt);
    }

    @Override
    protected void channelRead0(ChannelHandlerContext ctx, WsInPacket msg) {
        switch (msg) {
            case ClientConnected(var clientId) -> {
                for (DaianaListener listener : listeners) {
                    listener.onClientConnected(clientId);
                }
            }

            case ClientDisconnected(var clientId) -> {
                for (DaianaListener listener : listeners) {
                    listener.onClientDisconnected(clientId);
                }
            }

            case Message(var clientId, var payload) -> {
                for (DaianaListener listener : listeners) {
                    listener.onMessage(clientId, payload);
                }
            }

            case ServerInfo(var message) -> {
                for (DaianaListener listener : listeners) {
                    listener.onServerInfo(message);
                }
            }
        }
    }

    @Override
    public void channelInactive(ChannelHandlerContext ctx) throws Exception {
        for (DaianaListener listener : listeners) {
            listener.onDisconnected();
        }

        super.channelInactive(ctx);
    }

    @Override
    public void exceptionCaught(ChannelHandlerContext ctx, Throwable cause) throws Exception {
        for (DaianaListener listener : listeners) {
            listener.onError(cause);
        }

        if (!handshakeFuture.isDone()) {
            handshakeFuture.completeExceptionally(cause);
        }

        super.exceptionCaught(ctx, cause);
    }
}
