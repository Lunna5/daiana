package dev.lunna.daiana4j.protocol.out;

import dev.lunna.daiana4j.protocol.OpCodes;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

import java.util.UUID;

public record UnicastPacket(@NotNull UUID clientId, byte[] payload) implements WsOutPacket {
    @Override
    public @NotNull OpCodes.Client2Server opCode() {
        return OpCodes.Client2Server.UNICAST;
    }

    @Override
    public void writeToByteBuf(@NotNull ByteBuf buf) {
        buf.writeByte(opCode().getCode());
        buf.writeLong(clientId.getMostSignificantBits());
        buf.writeLong(clientId.getLeastSignificantBits());
        buf.writeBytes(payload);
    }
}