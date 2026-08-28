package dev.lunna.daiana4j.protocol.in;

import dev.lunna.daiana4j.protocol.OpCodes;
import io.netty.buffer.ByteBuf;
import io.netty.handler.codec.CorruptedFrameException;
import org.jetbrains.annotations.NotNull;

import static java.util.Objects.requireNonNull;

/**
 * Sealed interface representing an incoming packet received from the Daiana WebSocket server (Server -> Client).
 */
public sealed interface WsInPacket permits ClientConnected, ClientDisconnected, Message, ServerInfo {

    /**
     * Returns the server-to-client opcode of this packet.
     *
     * @return the {@link OpCodes.Server2Client} opcode
     */
    @NotNull OpCodes.Server2Client opCode();

    /**
     * Deserializes an incoming {@link ByteBuf} into a typed {@link WsInPacket} by reading the leading opcode.
     *
     * @param buf the raw Netty buffer containing the packet bytes
     * @return the deserialized {@link WsInPacket}
     * @throws CorruptedFrameException if the buffer is empty or the opcode is unrecognized
     */
    @NotNull
    static WsInPacket fromBytes(@NotNull ByteBuf buf) {
        requireNonNull(buf, "buf cannot be null");

        if (!buf.isReadable()) {
            throw new CorruptedFrameException("Empty buffer received, cannot read packet");
        }

        byte rawOpCode = buf.readByte();
        final var opcode = OpCodes.Server2Client.fromByte(rawOpCode);

        return switch (opcode) {
            case CLIENT_CONNECTED -> ClientConnected.fromByteBuf(buf);
            case CLIENT_DISCONNECTED -> ClientDisconnected.fromByteBuf(buf);
            case MESSAGE -> Message.fromByteBuf(buf);
            case SERVER_INFO -> ServerInfo.fromByteBuf(buf);
        };
    }
}
