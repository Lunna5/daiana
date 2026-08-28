package dev.lunna.daiana4j.protocol.out;

import dev.lunna.daiana4j.protocol.OpCodes;
import dev.lunna.daiana4j.util.UuidUtil;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

import java.util.UUID;

import static java.util.Objects.requireNonNull;

/**
 * Outbound packet to send a private direct message to a single destination peer (Opcode 0x00).
 * <p>
 * Binary structure: {@code [0x00 Opcode (1B)][16B Target UUID][Payload (NB)]}.
 *
 * @param clientId the destination peer's {@link UUID}
 * @param payload  the raw binary message
 */
public record UnicastPacket(@NotNull UUID clientId, @NotNull byte[] payload) implements WsOutPacket {

    /**
     * Compact constructor validating non-null arguments.
     *
     * @param clientId the destination peer's {@link UUID}
     * @param payload  the raw binary message
     */
    public UnicastPacket {
        requireNonNull(clientId, "clientId cannot be null");
        requireNonNull(payload, "payload cannot be null");
    }

    @Override
    public @NotNull OpCodes.Client2Server opCode() {
        return OpCodes.Client2Server.UNICAST;
    }

    @Override
    public void writeToByteBuf(@NotNull final ByteBuf buf) {
        requireNonNull(buf, "buf cannot be null");
        buf.writeByte(opCode().getCode());
        UuidUtil.writeUuidToByteBuf(buf, clientId);
        buf.writeBytes(payload);
    }
}