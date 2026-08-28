package dev.lunna.daiana4j.protocol.out;

import dev.lunna.daiana4j.protocol.OpCodes;
import dev.lunna.daiana4j.util.UuidUtil;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

import java.util.List;
import java.util.UUID;

import static java.util.Objects.requireNonNull;

/**
 * Outbound packet to send a targeted message to multiple specified peers in the room (Opcode 0x01).
 * <p>
 * Binary structure: {@code [0x01 Opcode (1B)][2B Count (u16)][N * 16B Target UUIDs][Payload (NB)]}.
 *
 * @param clientIds the list of destination peer {@link UUID}s
 * @param payload   the raw binary message
 */
public record MulticastPacket(@NotNull List<UUID> clientIds, @NotNull byte[] payload) implements WsOutPacket {

    /**
     * Compact constructor validating non-null arguments and client count limit.
     *
     * @param clientIds the list of destination peer {@link UUID}s
     * @param payload   the raw binary message
     */
    public MulticastPacket {
        requireNonNull(clientIds, "clientIds cannot be null");
        requireNonNull(payload, "payload cannot be null");
        if (clientIds.size() > 65535) {
            throw new IllegalArgumentException("Target client count cannot exceed 65535 (u16 limit)");
        }
    }

    @Override
    public @NotNull OpCodes.Client2Server opCode() {
        return OpCodes.Client2Server.MULTICAST;
    }

    @Override
    public void writeToByteBuf(@NotNull final ByteBuf buf) {
        requireNonNull(buf, "buf cannot be null");
        buf.writeByte(opCode().getCode());
        buf.writeShort(clientIds.size());
        for (UUID clientId : clientIds) {
            UuidUtil.writeUuidToByteBuf(buf, clientId);
        }
        buf.writeBytes(payload);
    }
}
