package dev.lunna.daiana4j.room;

import java.net.URI;
import java.net.URISyntaxException;
import java.util.UUID;

public record ServerRoomConnector(URI uri, UUID roomId) {
    public URI websocket() {
        try {
            return new URI(uri.toString()
                    .replaceFirst("https", "wss")
                    .replaceFirst("http", "ws") + "/room/" + roomId);
        } catch (URISyntaxException e) {
            throw new IllegalArgumentException("Invalid URI", e);
        }
    }
}
