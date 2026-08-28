package dev.lunna.daiana4j.room;


import java.util.List;
import java.util.UUID;
import java.util.concurrent.CopyOnWriteArrayList;

public final class RoomManager {
    private final List<UUID> clients = new CopyOnWriteArrayList<>();

    public void addClient(UUID clientId) {
        clients.add(clientId);
    }

    public void removeClient(UUID clientId) {
        clients.remove(clientId);
    }

    public List<UUID> getClients() {
        return clients;
    }
}
