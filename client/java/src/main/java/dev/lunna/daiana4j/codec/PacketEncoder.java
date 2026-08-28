package dev.lunna.daiana4j.codec;

import dev.lunna.daiana4j.protocol.out.WsOutPacket;
import io.netty.buffer.ByteBuf;
import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.MessageToMessageEncoder;
import io.netty.handler.codec.http.websocketx.BinaryWebSocketFrame;

import java.util.List;

/**
 * Netty outbound handler that encodes typed {@link WsOutPacket}s into {@link BinaryWebSocketFrame}s.
 */
public final class PacketEncoder extends MessageToMessageEncoder<WsOutPacket> {

    /**
     * Constructs a new {@link PacketEncoder}.
     */
    public PacketEncoder() {}

    @Override
    protected void encode(ChannelHandlerContext ctx, WsOutPacket msg, List<Object> out) {
        ByteBuf buf = ctx.alloc().buffer();
        msg.writeToByteBuf(buf);
        out.add(new BinaryWebSocketFrame(buf));
    }
}
