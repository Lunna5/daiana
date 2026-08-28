package dev.lunna.daiana4j.protocol;

/**
 * Protocol operation codes (Opcodes) for the Daiana binary WebSocket protocol.
 */
public final class OpCodes {
    private OpCodes() {}

    /**
     * Opcodes received from the server (Server -> Client).
     */
    public enum Server2Client {
        /** Opcode 0x00: A peer connected to the room. Payload contains the peer's 16-byte UUID. */
        CLIENT_CONNECTED((byte) 0x00),

        /** Opcode 0x01: A peer disconnected from the room. Payload contains the peer's 16-byte UUID. */
        CLIENT_DISCONNECTED((byte) 0x01),

        /** Opcode 0x02: Message from a peer. Payload contains 16-byte sender UUID + raw message bytes. */
        MESSAGE((byte) 0x02),

        /** Opcode 0x03: Server system/info message. Payload contains UTF-8 text bytes. */
        SERVER_INFO((byte) 0x03);

        private final byte code;

        Server2Client(byte code) {
            this.code = code;
        }

        /**
         * Returns the raw byte value of the opcode.
         *
         * @return the byte opcode
         */
        public byte getCode() {
            return code;
        }

        /**
         * Resolves the {@link Server2Client} enum constant from its raw byte code.
         *
         * @param code the byte opcode to resolve
         * @return the matching {@link Server2Client} enum constant
         * @throws IllegalArgumentException if the opcode is unknown
         */
        public static Server2Client fromByte(byte code) {
            for (Server2Client opCode : values()) {
                if (opCode.code == code) {
                    return opCode;
                }
            }
            throw new IllegalArgumentException("Unknown Server2Client opcode: " + code);
        }
    }

    /**
     * Opcodes sent by the client to the server (Client -> Server).
     */
    public enum Client2Server {
        /** Opcode 0x00: Private message to a single destination UUID. Format: [0x00][16B Target UUID][Payload]. */
        UNICAST((byte) 0x00),

        /** Opcode 0x01: Targeted message to multiple destination UUIDs. Format: [0x01][2B Count][N * 16B UUIDs][Payload]. */
        MULTICAST((byte) 0x01),

        /** Opcode 0x02: Broadcast message to all peers in the room. Format: [0x02][Payload]. */
        BROADCAST((byte) 0x02);

        private final byte code;

        Client2Server(byte code) {
            this.code = code;
        }

        /**
         * Returns the raw byte value of the opcode.
         *
         * @return the byte opcode
         */
        public byte getCode() {
            return code;
        }

        /**
         * Resolves the {@link Client2Server} enum constant from its raw byte code.
         *
         * @param code the byte opcode to resolve
         * @return the matching {@link Client2Server} enum constant
         * @throws IllegalArgumentException if the opcode is unknown
         */
        public static Client2Server fromByte(byte code) {
            for (Client2Server opCode : values()) {
                if (opCode.code == code) {
                    return opCode;
                }
            }
            throw new IllegalArgumentException("Unknown Client2Server opcode: " + code);
        }
    }
}
