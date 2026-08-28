package dev.lunna.daiana4j.codec;

import dev.lunna.daiana4j.protocol.in.WsInPacket;
import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.MessageToMessageDecoder;
import io.netty.handler.codec.http.websocketx.BinaryWebSocketFrame;

import java.util.List;

public final class PacketDecoder extends MessageToMessageDecoder<BinaryWebSocketFrame> {
    @Override
    protected void decode(ChannelHandlerContext ctx, BinaryWebSocketFrame msg, List<Object> out) throws Exception {
        WsInPacket packet = WsInPacket.fromBytes(msg.content());
        out.add(packet);
    }
}
