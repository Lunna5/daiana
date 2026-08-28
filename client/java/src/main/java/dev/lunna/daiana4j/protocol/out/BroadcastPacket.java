package dev.lunna.daiana4j.protocol.out;

import dev.lunna.daiana4j.protocol.OpCodes;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

public final class BroadcastPacket extends BasePacket {
    public BroadcastPacket(byte[] payload) {
        super(payload);
    }

    @Override
    public @NotNull OpCodes.Client2Server opCode() {
        return OpCodes.Client2Server.BROADCAST;
    }
}
