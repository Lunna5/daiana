package dev.lunna.daiana4j.protocol.out;

import dev.lunna.daiana4j.protocol.OpCodes;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

public sealed interface WsOutPacket permits BroadcastPacket, MulticastPacket, UnicastPacket {
    @NotNull OpCodes.Client2Server opCode();

    void writeToByteBuf(@NotNull final ByteBuf buf);
}
