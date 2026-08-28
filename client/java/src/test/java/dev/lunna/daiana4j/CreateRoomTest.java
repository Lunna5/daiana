package dev.lunna.daiana4j;

import org.junit.jupiter.api.Test;

import java.net.URI;
import java.net.URISyntaxException;

public class CreateRoomTest {
    public static String INSTANCE_URL = System.getProperty("instance.url", "http://localhost:8080");

    @Test
    public void testCreateRoom() throws URISyntaxException {
        var roomId = DaianaClient.createRoom(new URI(INSTANCE_URL)).join().roomId();
        assert roomId != null;
    }
}
