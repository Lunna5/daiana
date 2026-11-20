package dev.lunna.daiana4j;

import dev.lunna.daiana4j.protocol.OpCodes;
import dev.lunna.daiana4j.protocol.in.TempCodeResponse;
import dev.lunna.daiana4j.protocol.in.WsInPacket;
import dev.lunna.daiana4j.protocol.out.CreateTempCodePacket;
import dev.lunna.daiana4j.util.UuidUtil;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import org.junit.jupiter.api.Test;

import java.util.UUID;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;

public class PacketCodecTest {

    @Test
    public void testEncodeCreateTempCodePacket() {
        ByteBuf buf = Unpooled.buffer();
        try {
            new CreateTempCodePacket().writeToByteBuf(buf);
            assertEquals(1, buf.readableBytes());
            assertEquals(OpCodes.Client2Server.CREATE_TEMP_CODE.getCode(), buf.readByte());
        } finally {
            buf.release();
        }
    }

    @Test
    public void testDecodeTempCodeResponsePacket() {
        UUID expected = UUID.fromString("c1f7b889-4e78-4389-9407-73d8b28cf998");
        ByteBuf buf = Unpooled.buffer();
        try {
            buf.writeByte(OpCodes.Server2Client.TEMP_CODE_RESPONSE.getCode());
            UuidUtil.writeUuidToByteBuf(buf, expected);

            WsInPacket packet = WsInPacket.fromBytes(buf);
            TempCodeResponse response = assertInstanceOf(TempCodeResponse.class, packet);
            assertEquals(expected, response.code());
        } finally {
            buf.release();
        }
    }
}
