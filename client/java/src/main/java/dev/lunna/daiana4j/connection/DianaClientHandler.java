package dev.lunna.daiana4j.connection;

import dev.lunna.daiana4j.listener.DaianaListener;
import dev.lunna.daiana4j.protocol.in.*;
import io.netty.channel.ChannelHandlerContext;
import io.netty.channel.SimpleChannelInboundHandler;
import io.netty.handler.codec.http.websocketx.WebSocketClientProtocolHandler;

import java.util.List;
import java.util.concurrent.CompletableFuture;

public final class DianaClientHandler extends SimpleChannelInboundHandler<WsInPacket> {
    private final List<DaianaListener> listeners;
    private final CompletableFuture<Void> handshakeFuture;

    public DianaClientHandler(List<DaianaListener> listeners, CompletableFuture<Void> handshakeFuture) {
        this.listeners = listeners;
        this.handshakeFuture = handshakeFuture;
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
    protected void channelRead0(ChannelHandlerContext ctx, WsInPacket msg) throws Exception {
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

            case Message(var clientId, var message) -> {
                for (DaianaListener listener : listeners) {
                    listener.onMessage(clientId, message);
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
