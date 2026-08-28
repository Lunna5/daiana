package dev.lunna.daiana4j.protocol.out;

import dev.lunna.daiana4j.protocol.OpCodes;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

public record BroadcastPacket(byte[] payload) implements WsOutPacket {
    @Override
    public @NotNull OpCodes.Client2Server opCode() {
        return OpCodes.Client2Server.BROADCAST;
    }

    @Override
    public void writeToByteBuf(@NotNull final io.netty.buffer.ByteBuf buf) {
        buf.writeByte(opCode().getCode());
        buf.writeBytes(payload);
    }
}
