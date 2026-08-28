package dev.lunna.daiana4j.protocol.out;

import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

public abstract sealed class BasePacket implements WsOutPacket permits BroadcastPacket, MulticastPacket, UnicastPacket {
    private final byte[] payload;

    public BasePacket(byte[] payload) {
        this.payload = payload;
    }

    public byte[] getPayload() {
        return payload;
    }

    @Override
    public void writeToByteBuf(@NotNull ByteBuf buf) {
        buf.writeByte(opCode().getCode());
        buf.writeBytes(payload);
    }
}
