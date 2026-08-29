package dev.lunna.daiana4j;

import io.netty.handler.ssl.SslContext;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.time.Duration;

import static java.util.Objects.requireNonNull;

/**
 * Configuration options for the {@link DaianaClient}.
 * <p>
 * Allows tuning Netty buffer limits, frame sizes, connection/handshake timeouts, heartbeat intervals, and SSL/TLS settings.
 */
public final class DaianaClientOptions {

    /** Default maximum HTTP aggregated content length (1 MiB). */
    public static final int DEFAULT_MAX_CONTENT_LENGTH = 1024 * 1024;

    /** Default connection timeout (10 seconds). */
    public static final Duration DEFAULT_CONNECTION_TIMEOUT = Duration.ofSeconds(10);

    /** @deprecated Use {@link #DEFAULT_CONNECTION_TIMEOUT} instead. */
    @Deprecated
    public static final Duration DEFAULT_CONECTION_TIMEOUT = DEFAULT_CONNECTION_TIMEOUT;

    /** Default WebSocket handshake timeout (10 seconds). */
    public static final Duration DEFAULT_HANDSHAKE_TIMEOUT = Duration.ofSeconds(10);

    /** Default heartbeat ping interval (25 seconds). */
    public static final Duration DEFAULT_HEARTBEAT_INTERVAL = Duration.ofSeconds(25);

    private int maxContentLength = DEFAULT_MAX_CONTENT_LENGTH;
    private int maxFramePayloadLength = DEFAULT_MAX_CONTENT_LENGTH;
    private Duration connectionTimeout = DEFAULT_CONNECTION_TIMEOUT;
    private Duration handshakeTimeout = DEFAULT_HANDSHAKE_TIMEOUT;
    private Duration heartbeatInterval = DEFAULT_HEARTBEAT_INTERVAL;
    private SslContext sslContext;

    /**
     * Constructs default {@link DaianaClientOptions}.
     */
    public DaianaClientOptions() {}

    /**
     * Creates a new instance of {@link DaianaClientOptions} with default settings.
     *
     * @return a new {@link DaianaClientOptions} instance
     */
    public static DaianaClientOptions create() {
        return new DaianaClientOptions();
    }

    /**
     * Returns the maximum HTTP content length in bytes for aggregation.
     *
     * @return the max content length in bytes
     */
    public int getMaxContentLength() {
        return maxContentLength;
    }

    /**
     * Sets the maximum HTTP content length in bytes for aggregation.
     *
     * @param maxContentLength the maximum content length in bytes
     * @return this options instance for chaining
     */
    public DaianaClientOptions setMaxContentLength(int maxContentLength) {
        this.maxContentLength = maxContentLength;
        return this;
    }

    /**
     * Returns the maximum WebSocket frame payload length in bytes.
     *
     * @return the maximum frame payload length in bytes
     */
    public int getMaxFramePayloadLength() {
        return maxFramePayloadLength;
    }

    /**
     * Sets the maximum WebSocket frame payload length in bytes.
     *
     * @param maxFramePayloadLength the maximum frame payload length in bytes
     * @return this options instance for chaining
     */
    public DaianaClientOptions setMaxFramePayloadLength(int maxFramePayloadLength) {
        this.maxFramePayloadLength = maxFramePayloadLength;
        return this;
    }

    /**
     * Returns the TCP connection establishment timeout.
     *
     * @return the connection timeout
     */
    public Duration getConnectionTimeout() {
        return connectionTimeout;
    }

    /**
     * Sets the TCP connection establishment timeout.
     *
     * @param connectionTimeout the connection timeout
     * @return this options instance for chaining
     */
    public DaianaClientOptions setConnectionTimeout(@NotNull Duration connectionTimeout) {
        requireNonNull(connectionTimeout, "connectionTimeout cannot be null");
        this.connectionTimeout = connectionTimeout;
        return this;
    }

    /**
     * Returns the WebSocket handshake timeout.
     *
     * @return the handshake timeout
     */
    public Duration getHandshakeTimeout() {
        return handshakeTimeout;
    }

    /**
     * Sets the WebSocket handshake timeout.
     *
     * @param handshakeTimeout the handshake timeout
     * @return this options instance for chaining
     */
    public DaianaClientOptions setHandshakeTimeout(@NotNull Duration handshakeTimeout) {
        requireNonNull(handshakeTimeout, "handshakeTimeout cannot be null");
        this.handshakeTimeout = handshakeTimeout;
        return this;
    }

    /**
     * Returns the heartbeat ping interval.
     *
     * @return the heartbeat interval (or {@link Duration#ZERO} if disabled)
     */
    public Duration getHeartbeatInterval() {
        return heartbeatInterval;
    }

    /**
     * Sets the heartbeat ping interval to keep WebSocket connections alive through proxies.
     * Use {@link Duration#ZERO} to disable automatic heartbeats.
     *
     * @param heartbeatInterval the heartbeat interval
     * @return this options instance for chaining
     */
    public DaianaClientOptions setHeartbeatInterval(@NotNull Duration heartbeatInterval) {
        requireNonNull(heartbeatInterval, "heartbeatInterval cannot be null");
        this.heartbeatInterval = heartbeatInterval;
        return this;
    }

    /**
     * Returns the custom {@link SslContext}, or {@code null} if default client SSL context should be used for {@code wss://}.
     *
     * @return the configured {@link SslContext} or {@code null}
     */
    public @Nullable SslContext getSslContext() {
        return sslContext;
    }

    /**
     * Sets a custom {@link SslContext} to be used for secure WebSocket ({@code wss://}) connections.
     *
     * @param sslContext the custom {@link SslContext}
     * @return this options instance for chaining
     */
    public DaianaClientOptions setSslContext(@Nullable SslContext sslContext) {
        this.sslContext = sslContext;
        return this;
    }
}
