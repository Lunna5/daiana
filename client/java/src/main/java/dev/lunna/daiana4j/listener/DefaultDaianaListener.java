package dev.lunna.daiana4j.listener;

import dev.lunna.daiana4j.room.RoomManager;
import org.jetbrains.annotations.NotNull;

import java.util.UUID;

import static java.util.Objects.requireNonNull;

/**
 * Default implementation of {@link DaianaListener} that automatically tracks peer membership
 * in the provided {@link RoomManager}.
 * <p>
 * Registers connecting clients into the room list and removes disconnecting clients.
 */
public final class DefaultDaianaListener implements DaianaListener {
    private final RoomManager roomManager;

    /**
     * Constructs a new {@link DefaultDaianaListener}.
     *
     * @param roomManager the {@link RoomManager} instance to update
     */
    public DefaultDaianaListener(@NotNull RoomManager roomManager) {
        this.roomManager = requireNonNull(roomManager, "roomManager cannot be null");
    }

    @Override
    public void onClientConnected(@NotNull UUID clientId) {
        roomManager.addClient(clientId);
    }

    @Override
    public void onClientDisconnected(@NotNull UUID clientId) {
        roomManager.removeClient(clientId);
    }
}
