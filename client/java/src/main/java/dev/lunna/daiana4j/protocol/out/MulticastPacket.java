package dev.lunna.daiana4j.protocol.out;

import dev.lunna.daiana4j.protocol.OpCodes;
import org.jetbrains.annotations.NotNull;

import java.util.List;
import java.util.UUID;

public record MulticastPacket(List<UUID> clientIds, byte[] payload) implements WsOutPacket {
    @Override
    public @NotNull OpCodes.Client2Server opCode() {
        return OpCodes.Client2Server.MULTICAST;
    }

    @Override
    public void writeToByteBuf(@NotNull final io.netty.buffer.ByteBuf buf) {
        buf.writeByte(opCode().getCode());
        buf.writeInt(clientIds.size());
        for (UUID clientId : clientIds) {
            buf.writeLong(clientId.getMostSignificantBits());
            buf.writeLong(clientId.getLeastSignificantBits());
        }
        buf.writeBytes(payload);
    }
}
