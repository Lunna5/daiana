package dev.lunna.daiana4j;

import dev.lunna.daiana4j.listener.DaianaListener;
import io.netty.channel.EventLoopGroup;
import org.jetbrains.annotations.NotNull;

import java.net.URI;
import java.util.ArrayList;
import java.util.List;

import static java.util.Objects.requireNonNull;

public final class DaianaClientBuilder {
    private URI serverUri;
    private List<DaianaListener> listeners = new ArrayList<>();
    private DaianaClientOptions options = new DaianaClientOptions();
    private EventLoopGroup sharedGroup;
    private boolean includeDefaultListeners = true;

    private DaianaClientBuilder() {
        // Private constructor to enforce the use of the static create() method
    }

    public static DaianaClientBuilder create() {
        return new DaianaClientBuilder();
    }

    public DaianaClientBuilder serverUri(@NotNull URI uri) {
        requireNonNull(uri, "Server URI cannot be null");
        this.serverUri = uri;
        return this;
    }

    public DaianaClientBuilder addListener(@NotNull DaianaListener listener) {
        requireNonNull(listener, "Listener cannot be null");
        this.listeners.add(listener);
        return this;
    }

    public DaianaClientBuilder options(@NotNull DaianaClientOptions options) {
        requireNonNull(options, "Options cannot be null");
        this.options = options;
        return this;
    }

    public DaianaClientBuilder sharedEventLoopGroup(@NotNull EventLoopGroup group) {
        requireNonNull(group, "Shared EventLoopGroup cannot be null");
        this.sharedGroup = group;
        return this;
    }

    public DaianaClientBuilder includeDefaultListeners(boolean include) {
        this.includeDefaultListeners = include;
        return this;
    }

    public DaianaClient build() {
        if (serverUri == null) {
            throw new IllegalStateException("Server URI must be set before building the client.");
        }

        return new DaianaClient(serverUri, listeners, options, sharedGroup, includeDefaultListeners);
    }
}
