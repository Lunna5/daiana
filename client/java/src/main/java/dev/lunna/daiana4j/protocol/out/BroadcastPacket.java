package dev.lunna.daiana4j.protocol.out;

import dev.lunna.daiana4j.protocol.OpCodes;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

import static java.util.Objects.requireNonNull;

/**
 * Outbound packet to broadcast a message to all other connected peers in the room (Opcode 0x02).
 * <p>
 * Binary structure: {@code [0x02 Opcode (1B)][Payload (NB)]}.
 *
 * @param payload the raw binary message to broadcast
 */
public record BroadcastPacket(@NotNull byte[] payload) implements WsOutPacket {

    /**
     * Compact constructor validating non-null arguments.
     *
     * @param payload the raw binary message to broadcast
     */
    public BroadcastPacket {
        requireNonNull(payload, "payload cannot be null");
    }

    @Override
    public @NotNull OpCodes.Client2Server opCode() {
        return OpCodes.Client2Server.BROADCAST;
    }

    @Override
    public void writeToByteBuf(@NotNull final ByteBuf buf) {
        requireNonNull(buf, "buf cannot be null");
        buf.writeByte(opCode().getCode());
        buf.writeBytes(payload);
    }
}
