package dev.lunna.daiana4j.protocol.in;

import dev.lunna.daiana4j.protocol.OpCodes;
import dev.lunna.daiana4j.util.UuidUtil;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

import java.util.UUID;

import static java.util.Objects.requireNonNull;

/**
 * Inbound packet notifying that a peer disconnected from the room (Opcode 0x01).
 *
 * @param clientId the {@link UUID} of the peer that left
 */
public record ClientDisconnected(@NotNull UUID clientId) implements WsInPacket {

    /**
     * Compact constructor validating non-null arguments.
     *
     * @param clientId the {@link UUID} of the peer that left
     */
    public ClientDisconnected {
        requireNonNull(clientId, "clientId cannot be null");
    }

    @Override
    public @NotNull OpCodes.Server2Client opCode() {
        return OpCodes.Server2Client.CLIENT_DISCONNECTED;
    }

    /**
     * Deserializes a {@link ClientDisconnected} packet from a {@link ByteBuf}.
     *
     * @param buf the buffer to read from
     * @return the deserialized {@link ClientDisconnected} packet
     */
    @NotNull
    public static ClientDisconnected fromByteBuf(@NotNull final ByteBuf buf) {
        return new ClientDisconnected(UuidUtil.fromByteBuf(buf));
    }
}
