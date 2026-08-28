package dev.lunna.daiana4j.protocol.in;

import dev.lunna.daiana4j.protocol.OpCodes;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

import java.nio.charset.StandardCharsets;

import static java.util.Objects.requireNonNull;

public record ServerInfo(String message) implements WsInPacket {
    public ServerInfo {
        requireNonNull(message, "message cannot be null");
    }

    @Override
    public OpCodes.Server2Client opCode() {
        return OpCodes.Server2Client.SERVER_INFO;
    }

    @NotNull
    public static ServerInfo fromByteBuf(@NotNull final ByteBuf buf) {
        requireNonNull(buf, "buf cannot be null");
        String text = buf.toString(StandardCharsets.UTF_8);
        return new ServerInfo(new String(text));
    }
}
