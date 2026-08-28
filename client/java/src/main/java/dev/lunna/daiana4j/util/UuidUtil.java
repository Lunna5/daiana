package dev.lunna.daiana4j.util;

import io.netty.buffer.ByteBuf;
import io.netty.handler.codec.CorruptedFrameException;
import org.jetbrains.annotations.NotNull;

import java.util.UUID;

import static java.util.Objects.requireNonNull;

/**
 * Utility functions for reading and writing 16-byte raw {@link UUID}s to Netty {@link ByteBuf}s.
 */
public final class UuidUtil {
    private UuidUtil() {
        throw new UnsupportedOperationException("Utility class cannot be instantiated");
    }

    /**
     * Reads 16 bytes from the {@link ByteBuf} and constructs a {@link UUID} from the most and least significant bits.
     *
     * @param buf the buffer to read from
     * @return the reconstructed {@link UUID}
     * @throws CorruptedFrameException if the buffer has fewer than 16 readable bytes
     */
    @NotNull
    public static UUID fromByteBuf(@NotNull ByteBuf buf) {
        requireNonNull(buf, "buf cannot be null");

        if (buf.readableBytes() < 16) {
            throw new CorruptedFrameException("Incomplete UUID bytes: expected at least 16 bytes, found " + buf.readableBytes());
        }

        long mostSigBits = buf.readLong();
        long leastSigBits = buf.readLong();

        return new UUID(mostSigBits, leastSigBits);
    }

    /**
     * Writes the 16 bytes of a {@link UUID} (most and least significant bits) to a {@link ByteBuf}.
     *
     * @param buf  the buffer to write to
     * @param uuid the {@link UUID} to write
     */
    public static void writeUuidToByteBuf(@NotNull ByteBuf buf, @NotNull UUID uuid) {
        requireNonNull(buf, "buf cannot be null");
        requireNonNull(uuid, "uuid cannot be null");

        buf.writeLong(uuid.getMostSignificantBits());
        buf.writeLong(uuid.getLeastSignificantBits());
    }
}
