package dev.lunna.daiana4j.room;

import org.jetbrains.annotations.NotNull;

import java.util.Collections;
import java.util.List;
import java.util.UUID;
import java.util.concurrent.CopyOnWriteArrayList;

import static java.util.Objects.requireNonNull;

/**
 * Thread-safe manager for tracking the list of active connected peers in a room.
 */
public final class RoomManager {
    private final List<UUID> clients = new CopyOnWriteArrayList<>();

    /**
     * Constructs a new empty {@link RoomManager}.
     */
    public RoomManager() {}

    /**
     * Adds a peer {@link UUID} to the room's active client list.
     *
     * @param clientId the {@link UUID} of the peer to add
     */
    public void addClient(@NotNull UUID clientId) {
        requireNonNull(clientId, "clientId cannot be null");
        clients.add(clientId);
    }

    /**
     * Removes a peer {@link UUID} from the room's active client list.
     *
     * @param clientId the {@link UUID} of the peer to remove
     */
    public void removeClient(@NotNull UUID clientId) {
        requireNonNull(clientId, "clientId cannot be null");
        clients.remove(clientId);
    }

    /**
     * Returns an unmodifiable snapshot list of currently connected peer {@link UUID}s.
     *
     * @return the list of client {@link UUID}s
     */
    public @NotNull List<UUID> getClients() {
        return Collections.unmodifiableList(clients);
    }
}
