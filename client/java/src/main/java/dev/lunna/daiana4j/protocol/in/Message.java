package dev.lunna.daiana4j.protocol.in;

import dev.lunna.daiana4j.protocol.OpCodes;
import dev.lunna.daiana4j.util.UuidUtil;
import io.netty.buffer.ByteBuf;
import org.jetbrains.annotations.NotNull;

import java.util.Arrays;
import java.util.UUID;

import static java.util.Objects.requireNonNull;

public record Message(@NotNull UUID senderId, byte[] payload) implements WsInPacket {
    public Message {
        requireNonNull(senderId, "senderId cannot be null");
        requireNonNull(payload, "payload cannot be null");
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (!(obj instanceof Message message)) return false;
        return senderId.equals(message.senderId) && Arrays.equals(payload, message.payload);

    }

    @Override
    public int hashCode() {
        int result = senderId.hashCode();
        result = 31 * result + Arrays.hashCode(payload);
        return result;
    }

    @Override
    public @NotNull String toString() {
        return "Message[senderId=" + senderId + ", payloadLength=" + payload.length + "]";
    }

    @Override
    public OpCodes.Server2Client opCode() {
        return OpCodes.Server2Client.MESSAGE;
    }

    public static Message fromByteBuf(@NotNull final ByteBuf buf) {
        requireNonNull(buf, "buf cannot be null");
        UUID senderId = UuidUtil.fromByteBuf(buf);

        byte[] payload = new byte[buf.readableBytes()];
        buf.readBytes(payload);
        return new Message(senderId, payload);
    }
}
