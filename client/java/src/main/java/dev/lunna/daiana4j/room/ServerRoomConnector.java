package dev.lunna.daiana4j.room;

import org.jetbrains.annotations.NotNull;

import java.net.URI;
import java.net.URISyntaxException;
import java.util.UUID;

import static java.util.Objects.requireNonNull;

/**
 * Holder representing a created room on a Daiana server, containing the base HTTP URI and room {@link UUID}.
 *
 * @param uri    the base HTTP URI of the server
 * @param roomId the created room's {@link UUID}
 */
public record ServerRoomConnector(@NotNull URI uri, @NotNull UUID roomId) {

    /**
     * Compact constructor validating non-null arguments.
     *
     * @param uri    the base HTTP URI of the server
     * @param roomId the created room's {@link UUID}
     */
    public ServerRoomConnector {
        requireNonNull(uri, "uri cannot be null");
        requireNonNull(roomId, "roomId cannot be null");
    }

    /**
     * Derives the corresponding WebSocket URI ({@code ws://} or {@code wss://}) for connecting to this room.
     *
     * @return the WebSocket URI pointing to {@code /room/{roomId}}
     */
    public @NotNull URI websocket() {
        try {
            String base = uri.toString().replaceAll("/+$", "");
            String wsBase = base.replaceFirst("^http", "ws");
            return new URI(wsBase + "/room/" + roomId);
        } catch (URISyntaxException e) {
            throw new IllegalArgumentException("Invalid URI syntax", e);
        }
    }
}
