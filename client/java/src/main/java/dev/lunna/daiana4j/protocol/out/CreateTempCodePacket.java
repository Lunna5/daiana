package dev.lunna.daiana4j.protocol.out;

import dev.lunna.daiana4j.protocol.OpCodes;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

import static java.util.Objects.requireNonNull;

/**
 * Outbound packet requesting the server to generate a temporary room join code (Opcode 0x03).
 * <p>
 * Binary structure: {@code [0x03 Opcode (1B)]}.
 */
public record CreateTempCodePacket() implements WsOutPacket {

    @Override
    public @NotNull OpCodes.Client2Server opCode() {
        return OpCodes.Client2Server.CREATE_TEMP_CODE;
    }

    @Override
    public void writeToByteBuf(@NotNull final ByteBuf buf) {
        requireNonNull(buf, "buf cannot be null");
        buf.writeByte(opCode().getCode());
    }
}
