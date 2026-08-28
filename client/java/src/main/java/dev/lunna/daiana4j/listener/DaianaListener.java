package dev.lunna.daiana4j.listener;

import org.jetbrains.annotations.NotNull;

import java.util.UUID;

/**
 * Event listener interface for receiving connection lifecycle events and incoming binary packets
 * from the Daiana WebSocket relay server.
 * <p>
 * All methods have default empty implementations, allowing implementers to override only the events
 * they are interested in.
 */
public interface DaianaListener {

    /**
     * Invoked when the WebSocket connection and handshake have successfully completed.
     */
    default void onConnected() {}

    /**
     * Invoked when the WebSocket connection has been closed or dropped.
     */
    default void onDisconnected() {}

    /**
     * Invoked when a peer connects to the room (Server Opcode 0x00).
     *
     * @param clientId the {@link UUID} of the newly connected peer
     */
    default void onClientConnected(@NotNull UUID clientId) {}

    /**
     * Invoked when a peer disconnects from the room (Server Opcode 0x01).
     *
     * @param clientId the {@link UUID} of the disconnected peer
     */
    default void onClientDisconnected(@NotNull UUID clientId) {}

    /**
     * Invoked when a binary message packet is received from a peer (Server Opcode 0x02).
     *
     * @param senderId the verified {@link UUID} of the message sender assigned by the server
     * @param payload  the raw byte payload sent by the peer
     */
    default void onMessage(@NotNull UUID senderId, @NotNull byte[] payload) {}

    /**
     * Invoked when the server sends an administrative or informational system message (Server Opcode 0x03).
     *
     * @param message the informational text message sent by the server
     */
    default void onServerInfo(@NotNull String message) {}

    /**
     * Invoked when an unhandled exception or I/O error occurs in the Netty channel pipeline.
     *
     * @param throwable the error or exception caught
     */
    default void onError(@NotNull Throwable throwable) {}
}
