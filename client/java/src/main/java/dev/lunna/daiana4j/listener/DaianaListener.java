package dev.lunna.daiana4j.listener;

import org.jetbrains.annotations.NotNull;

import java.util.UUID;

public interface DaianaListener {
    default void onConnected() {}

    default void onDisconnected() {}

    default void onClientConnected(@NotNull UUID clientId) {}

    default void onClientDisconnected(@NotNull UUID clientId) {}

    default void onMessage(@NotNull UUID senderId, @NotNull byte[] payload) {}

    default void onServerInfo(@NotNull String message) {}

    default void onError(@NotNull Throwable throwable) {}
}
