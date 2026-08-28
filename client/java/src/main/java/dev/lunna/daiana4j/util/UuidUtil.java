package dev.lunna.daiana4j.util;

import io.netty.buffer.ByteBuf;
import io.netty.handler.codec.CorruptedFrameException;
import org.jetbrains.annotations.NotNull;

import java.util.UUID;

import static java.util.Objects.requireNonNull;

public final class UuidUtil {
    private UuidUtil() {
        throw new UnsupportedOperationException("Utility class");
    }

    @NotNull
    public static UUID fromByteBuf(@NotNull ByteBuf buf) {
        requireNonNull(buf, "buf cannot be null");

        if (buf.readableBytes() < 16) {
            throw new CorruptedFrameException("Incomplete UUID in ClientConnected packet");
        }

        long mostSigBits = buf.readLong();
        long leastSigBits = buf.readLong();

        return new UUID(mostSigBits, leastSigBits);
    }

    public static void writeUuidToByteBuf(@NotNull ByteBuf buf, @NotNull UUID uuid) {
        requireNonNull(buf, "buf cannot be null");
        requireNonNull(uuid, "uuid cannot be null");

        buf.writeLong(uuid.getMostSignificantBits());
        buf.writeLong(uuid.getLeastSignificantBits());
    }
}
