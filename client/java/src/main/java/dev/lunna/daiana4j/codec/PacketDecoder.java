package dev.lunna.daiana4j.codec;

import dev.lunna.daiana4j.protocol.in.WsInPacket;
import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.MessageToMessageDecoder;
import io.netty.handler.codec.http.websocketx.BinaryWebSocketFrame;

import java.util.List;

/**
 * Netty inbound handler that decodes incoming {@link BinaryWebSocketFrame}s into typed {@link WsInPacket}s.
 */
public final class PacketDecoder extends MessageToMessageDecoder<BinaryWebSocketFrame> {

    /**
     * Constructs a new {@link PacketDecoder}.
     */
    public PacketDecoder() {}

    @Override
    protected void decode(ChannelHandlerContext ctx, BinaryWebSocketFrame msg, List<Object> out) {
        WsInPacket packet = WsInPacket.fromBytes(msg.content());
        out.add(packet);
    }
}
