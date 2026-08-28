package dev.lunna.daiana4j.listener;

import dev.lunna.daiana4j.room.RoomManager;
import org.jetbrains.annotations.NotNull;

import java.util.UUID;

public final class DefaultDaianaListener implements DaianaListener {
    private final RoomManager roomManager;

    public DefaultDaianaListener(RoomManager roomManager) {
        this.roomManager = roomManager;
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
