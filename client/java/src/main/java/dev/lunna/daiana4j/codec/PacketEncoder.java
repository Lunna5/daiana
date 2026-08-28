package dev.lunna.daiana4j.codec;

import dev.lunna.daiana4j.protocol.out.WsOutPacket;
import io.netty.buffer.ByteBuf;
import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.MessageToMessageEncoder;
import io.netty.handler.codec.http.websocketx.BinaryWebSocketFrame;

import java.util.List;

public final class PacketEncoder extends MessageToMessageEncoder<WsOutPacket> {

    @Override
    protected void encode(ChannelHandlerContext ctx, WsOutPacket msg, List<Object> out) throws Exception {
        ByteBuf buf = ctx.alloc().buffer();
        msg.writeToByteBuf(buf);
        out.add(new BinaryWebSocketFrame(buf));
    }
}
