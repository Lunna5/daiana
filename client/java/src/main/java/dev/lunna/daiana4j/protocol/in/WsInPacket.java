package dev.lunna.daiana4j.protocol.in;

import dev.lunna.daiana4j.protocol.OpCodes;
import io.netty.buffer.ByteBuf;
import io.netty.handler.codec.CorruptedFrameException;
import org.jetbrains.annotations.NotNull;

import static java.util.Objects.requireNonNull;

public sealed interface WsInPacket permits ClientConnected, ClientDisconnected, Message, ServerInfo {
    @NotNull OpCodes.Server2Client opCode();

    @NotNull
    static WsInPacket fromBytes(@NotNull ByteBuf buf) {
        requireNonNull(buf, "buf cannot be null");

        if(!buf.isReadable()) {
            throw new CorruptedFrameException("Empty buffer received, cannot read packet");
        }

        byte rawOpCode = buf.readByte();
        final var opcode = OpCodes.Server2Client.fromByte(rawOpCode);

        return switch (opcode) {
            case CLIENT_CONNECTED -> ClientConnected.fromByteBuf(buf);
            case CLIENT_DISCONNECTED -> ClientDisconnected.fromByteBuf(buf);
            case MESSAGE -> Message.fromByteBuf(buf);
            case SERVER_INFO -> ServerInfo.fromByteBuf(buf);
            default -> throw new CorruptedFrameException("Unknown opcode: " + rawOpCode);
        };
    }
}
