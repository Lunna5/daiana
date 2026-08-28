package dev.lunna.daiana4j.protocol.in;

import dev.lunna.daiana4j.protocol.OpCodes;
import dev.lunna.daiana4j.util.UuidUtil;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

import java.util.UUID;

import static java.util.Objects.requireNonNull;

public record ClientConnected(@NotNull UUID clientId) implements WsInPacket {
    public ClientConnected {
        requireNonNull(clientId, "clientId cannot be null");
    }

    @Override
    public OpCodes.Server2Client opCode() {
        return OpCodes.Server2Client.CLIENT_CONNECTED;
    }

    @NotNull
    public static ClientConnected fromByteBuf(@NotNull final ByteBuf buf) {
        return new ClientConnected(UuidUtil.fromByteBuf(buf));
    }
}
