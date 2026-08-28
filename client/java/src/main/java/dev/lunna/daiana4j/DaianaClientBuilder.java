package dev.lunna.daiana4j;

import dev.lunna.daiana4j.listener.DaianaListener;
import io.netty.channel.EventLoopGroup;
import org.jetbrains.annotations.NotNull;

import java.net.URI;
import java.util.ArrayList;
import java.util.List;

import static java.util.Objects.requireNonNull;

/**
 * Fluent builder for configuring and creating instances of {@link DaianaClient}.
 *
 * <p>Example:
 * <pre>{@code
 * DaianaClient client = DaianaClientBuilder.create()
 *         .serverUri(URI.create("ws://localhost:8080/room/" + roomId))
 *         .addListener(myListener)
 *         .options(DaianaClientOptions.create().setMaxContentLength(2 * 1024 * 1024))
 *         .build();
 * }</pre>
 */
public final class DaianaClientBuilder {
    private URI serverUri;
    private final List<DaianaListener> listeners = new ArrayList<>();
    private DaianaClientOptions options = new DaianaClientOptions();
    private EventLoopGroup sharedGroup;
    private boolean includeDefaultListeners = true;

    private DaianaClientBuilder() {
        // Enforce usage of create() factory method
    }

    /**
     * Creates a new instance of {@link DaianaClientBuilder}.
     *
     * @return a fresh {@link DaianaClientBuilder}
     */
    public static DaianaClientBuilder create() {
        return new DaianaClientBuilder();
    }

    /**
     * Sets the target WebSocket URI of the room.
     *
     * @param uri the WebSocket URI (e.g. {@code ws://host:port/room/{roomId}})
     * @return this builder for chaining
     */
    public DaianaClientBuilder serverUri(@NotNull URI uri) {
        requireNonNull(uri, "Server URI cannot be null");
        this.serverUri = uri;
        return this;
    }

    /**
     * Registers an event listener to receive callbacks for connection events and packet reception.
     *
     * @param listener the {@link DaianaListener} to register
     * @return this builder for chaining
     */
    public DaianaClientBuilder addListener(@NotNull DaianaListener listener) {
        requireNonNull(listener, "Listener cannot be null");
        this.listeners.add(listener);
        return this;
    }

    /**
     * Sets the client configuration options.
     *
     * @param options the {@link DaianaClientOptions} configuration
     * @return this builder for chaining
     */
    public DaianaClientBuilder options(@NotNull DaianaClientOptions options) {
        requireNonNull(options, "Options cannot be null");
        this.options = options;
        return this;
    }

    /**
     * Configures a shared Netty {@link EventLoopGroup}.
     * <p>
     * If provided, {@link DaianaClient} will reuse this event loop and will NOT shut it down
     * upon calling {@link DaianaClient#disconnect()} or {@link DaianaClient#close()}.
     *
     * @param group the shared {@link EventLoopGroup}
     * @return this builder for chaining
     */
    public DaianaClientBuilder sharedEventLoopGroup(@NotNull EventLoopGroup group) {
        requireNonNull(group, "Shared EventLoopGroup cannot be null");
        this.sharedGroup = group;
        return this;
    }

    /**
     * Configures whether to automatically register the {@link dev.lunna.daiana4j.listener.DefaultDaianaListener}
     * for tracking connected peers in {@link dev.lunna.daiana4j.room.RoomManager}.
     *
     * @param include {@code true} to include default listeners (default: {@code true}), {@code false} otherwise
     * @return this builder for chaining
     */
    public DaianaClientBuilder includeDefaultListeners(boolean include) {
        this.includeDefaultListeners = include;
        return this;
    }

    /**
     * Builds and returns a new configured {@link DaianaClient}.
     *
     * @return the newly constructed {@link DaianaClient}
     * @throws IllegalStateException if {@link #serverUri(URI)} was not set
     */
    public DaianaClient build() {
        if (serverUri == null) {
            throw new IllegalStateException("Server URI must be set before building the client.");
        }

        return new DaianaClient(serverUri, listeners, options, sharedGroup, includeDefaultListeners);
    }
}
