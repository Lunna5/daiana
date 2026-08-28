package dev.lunna.daiana4j.protocol.out;

import dev.lunna.daiana4j.protocol.OpCodes;
import org.jetbrains.annotations.NotNull;

public final class UnicastPacket extends BasePacket {
    public UnicastPacket(byte[] payload) {
        super(payload);
    }

    @Override
    public @NotNull OpCodes.Client2Server opCode() {
        return OpCodes.Client2Server.UNICAST;
    }
}
