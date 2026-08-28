package dev.lunna.daiana4j.protocol.in;

import dev.lunna.daiana4j.protocol.OpCodes;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

import java.nio.charset.StandardCharsets;

import static java.util.Objects.requireNonNull;

/**
 * Inbound packet containing an informational text message from the server (Opcode 0x03).
 *
 * @param message the UTF-8 text message
 */
public record ServerInfo(@NotNull String message) implements WsInPacket {

    /**
     * Compact constructor validating non-null arguments.
     *
     * @param message the UTF-8 text message
     */
    public ServerInfo {
        requireNonNull(message, "message cannot be null");
    }

    @Override
    public @NotNull OpCodes.Server2Client opCode() {
        return OpCodes.Server2Client.SERVER_INFO;
    }

    /**
     * Deserializes a {@link ServerInfo} packet from a {@link ByteBuf}.
     *
     * @param buf the buffer containing the UTF-8 encoded text
     * @return the deserialized {@link ServerInfo} packet
     */
    @NotNull
    public static ServerInfo fromByteBuf(@NotNull final ByteBuf buf) {
        requireNonNull(buf, "buf cannot be null");
        String text = buf.toString(StandardCharsets.UTF_8);
        return new ServerInfo(text);
    }
}
