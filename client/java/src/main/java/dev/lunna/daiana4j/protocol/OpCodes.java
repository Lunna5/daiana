package dev.lunna.daiana4j.protocol;

public final class OpCodes {
    private OpCodes() {}

    public enum Server2Client {
        CLIENT_CONNECTED((byte) 0x00),
        CLIENT_DISCONNECTED((byte) 0x01),
        MESSAGE((byte) 0x02),
        SERVER_INFO((byte) 0x03);

        private final byte code;

        Server2Client(byte code) {
            this.code = code;
        }

        public byte getCode() {
            return code;
        }

        public static Server2Client fromByte(byte code) {
            for (Server2Client opCode : values()) {
                if (opCode.code == code) {
                    return opCode;
                }
            }
            throw new IllegalArgumentException("Unknown Server2Client opcode: " + code);
        }
    }

    public enum Client2Server {
        UNICAST((byte) 0x00),
        MULTICAST((byte) 0x01),
        BROADCAST((byte) 0x02);

        private final byte code;

        Client2Server(byte code) {
            this.code = code;
        }

        public byte getCode() {
            return code;
        }

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
