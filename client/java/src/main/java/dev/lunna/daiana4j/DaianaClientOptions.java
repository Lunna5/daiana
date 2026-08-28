package dev.lunna.daiana4j;

import java.time.Duration;

public final class DaianaClientOptions {
    public static final int DEFAULT_MAX_CONTENT_LENGTH = 1024 * 1024; // 1 MB
    public static final Duration DEFAULT_CONECTION_TIMEOUT = Duration.ofSeconds(10);
    public static final Duration DEFAULT_HANDSHAKE_TIMEOUT = Duration.ofSeconds(10);

    private int maxContentLenght = DEFAULT_MAX_CONTENT_LENGTH;
    private int maxFramePayloadLength = DEFAULT_MAX_CONTENT_LENGTH;
    private Duration connectionTimeout = DEFAULT_CONECTION_TIMEOUT;
    private Duration handshakeTimeout = DEFAULT_HANDSHAKE_TIMEOUT;

    public static DaianaClientOptions create() {
        return new DaianaClientOptions();
    }

    public int getMaxContentLength() {
        return maxContentLenght;
    }

    public DaianaClientOptions setMaxContentLength(int maxContentLength) {
        this.maxContentLenght = maxContentLength;
        return this;
    }

    public int getMaxFramePayloadLength() {
        return maxFramePayloadLength;
    }

    public DaianaClientOptions setMaxFramePayloadLength(int maxFramePayloadLength) {
        this.maxFramePayloadLength = maxFramePayloadLength;
        return this;
    }

    public Duration getConnectionTimeout() {
        return connectionTimeout;
    }

    public DaianaClientOptions setConnectionTimeout(Duration connectionTimeout) {
        this.connectionTimeout = connectionTimeout;
        return this;
    }

    public Duration getHandshakeTimeout() {
        return handshakeTimeout;
    }

    public DaianaClientOptions setHandshakeTimeout(Duration handshakeTimeout) {
        this.handshakeTimeout = handshakeTimeout;
        return this;
    }
}
