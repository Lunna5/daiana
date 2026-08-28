package dev.lunna.daiana4j.protocol.out;

import dev.lunna.daiana4j.protocol.OpCodes;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

/**
 * Sealed interface representing an outbound packet to be sent to the Daiana WebSocket server (Client -> Server).
 */
public sealed interface WsOutPacket permits BroadcastPacket, MulticastPacket, UnicastPacket {

    /**
     * Returns the client-to-server opcode of this packet.
     *
     * @return the {@link OpCodes.Client2Server} opcode
     */
    @NotNull OpCodes.Client2Server opCode();

    /**
     * Serializes this packet into the provided Netty {@link ByteBuf}.
     *
     * @param buf the destination buffer
     */
    void writeToByteBuf(@NotNull final ByteBuf buf);
}
