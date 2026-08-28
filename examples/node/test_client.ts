/**
 * DAIANA - TypeScript WebSocket Test Client
 * Run with: npx tsx examples/test_client.ts
 */

const HTTP_URL = process.env.HTTP_URL || "http://127.0.0.1:8080";
const WS_URL = process.env.WS_URL || "ws://127.0.0.1:8080";

// Server -> Client Opcodes (WsPacket)
const OP_CONNECTED = 0x0;
const OP_DISCONNECTED = 0x1;
const OP_MESSAGE = 0x2;
const OP_SERVER_INFO = 0x3;

// Client -> Server Opcodes (WsInPacket)
const IN_OP_UNICAST = 0x0;
const IN_OP_MULTICAST = 0x1;
const IN_OP_BROADCAST = 0x2;

// UUID Helpers
function uuidToBytes(uuid: string): Uint8Array {
  const clean = uuid.replace(/-/g, "");
  const bytes = new Uint8Array(16);
  for (let i = 0; i < 16; i++) {
    bytes[i] = parseInt(clean.substring(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

function bytesToUuid(bytes: Uint8Array, offset = 0): string {
  const hex: string[] = [];
  for (let i = offset; i < offset + 16; i++) {
    hex.push(bytes[i].toString(16).padStart(2, "0"));
  }
  return [
    hex.slice(0, 4).join(""),
    hex.slice(4, 6).join(""),
    hex.slice(6, 8).join(""),
    hex.slice(8, 10).join(""),
    hex.slice(10, 16).join(""),
  ].join("-");
}

// Packet Encoders
function encodeBroadcast(payloadText: string): Uint8Array {
  const payload = new TextEncoder().encode(payloadText);
  const out = new Uint8Array(1 + payload.length);
  out[0] = IN_OP_BROADCAST;
  out.set(payload, 1);
  return out;
}

function encodeUnicast(targetUuid: string, payloadText: string): Uint8Array {
  const payload = new TextEncoder().encode(payloadText);
  const targetBytes = uuidToBytes(targetUuid);
  const out = new Uint8Array(1 + 16 + payload.length);
  out[0] = IN_OP_UNICAST;
  out.set(targetBytes, 1);
  out.set(payload, 17);
  return out;
}

function encodeMulticast(targetUuids: string[], payloadText: string): Uint8Array {
  const payload = new TextEncoder().encode(payloadText);
  const out = new Uint8Array(1 + 2 + targetUuids.length * 16 + payload.length);
  out[0] = IN_OP_MULTICAST;
  const view = new DataView(out.buffer);
  view.setUint16(1, targetUuids.length, false); // Big endian
  let offset = 3;
  for (const uuid of targetUuids) {
    out.set(uuidToBytes(uuid), offset);
    offset += 16;
  }
  out.set(payload, offset);
  return out;
}

// Packet Decoder
function decodePacket(buffer: ArrayBuffer) {
  const bytes = new Uint8Array(buffer);
  const opcode = bytes[0];

  switch (opcode) {
    case OP_CONNECTED:
      return { type: "ClientConnected", clientId: bytesToUuid(bytes, 1) };
    case OP_DISCONNECTED:
      return { type: "ClientDisconnected", clientId: bytesToUuid(bytes, 1) };
    case OP_MESSAGE:
      return {
        type: "Message",
        senderId: bytesToUuid(bytes, 1),
        payload: new TextDecoder().decode(bytes.slice(17)),
      };
    case OP_SERVER_INFO:
      return {
        type: "ServerInfo",
        message: new TextDecoder().decode(bytes.slice(1)),
      };
    default:
      return { type: "Unknown", opcode };
  }
}

async function main() {
  console.log("\x1b[1;36m========================================================\x1b[0m");
  console.log("\x1b[1;36m    DAIANA - TypeScript WebSocket Test Client          \x1b[0m");
  console.log("\x1b[1;36m========================================================\x1b[0m\n");

  // 1. Health check
  process.stdout.write(`🌱 [1/5] Checking server health at ${HTTP_URL}/... `);
  const healthRes = await fetch(`${HTTP_URL}/`);
  const health = await healthRes.json();
  // @ts-ignore
  console.log(`\x1b[32mOK\x1b[0m (ping: ${health.ping}, version: ${health.version})`);

  // 2. Create room
  process.stdout.write(`🌱 [2/5] Creating room via POST ${HTTP_URL}/room/... `);
  const roomRes = await fetch(`${HTTP_URL}/room/`, { method: "POST" });
  // @ts-ignore
  const { id: roomId } = await roomRes.json();
  console.log(`\x1b[32mOK\x1b[0m (Room UUID: \x1b[35m${roomId}\x1b[0m)`);

  // 3. Connect WebSockets (Alice and Bob)
  const WebSocketImpl = globalThis.WebSocket;
  if (!WebSocketImpl) {
    throw new Error("WebSocket is not available in this Node runtime. Please run with Node 22+ or Bun.");
  }

  process.stdout.write(`🌱 [3/5] Connecting clients to ${WS_URL}/room/${roomId}... `);
  const alice = new WebSocketImpl(`${WS_URL}/room/${roomId}`);
  alice.binaryType = "arraybuffer";

  await new Promise((resolve) => alice.addEventListener("open", resolve, { once: true }));

  const bob = new WebSocketImpl(`${WS_URL}/room/${roomId}`);
  bob.binaryType = "arraybuffer";
  await new Promise((resolve) => bob.addEventListener("open", resolve, { once: true }));

  console.log(`\x1b[32mCONNECTED\x1b[0m`);

  // 4. Capture initial connection events
  let bobId = "";
  let aliceId = "";

  alice.addEventListener("message", (event) => {
    const packet = decodePacket(event.data as ArrayBuffer);
    if (packet.type === "ClientConnected") {
      bobId = packet.clientId || "";
      console.log(`   🦋 [Alice] Received ClientConnected event -> Bob: \x1b[33m${bobId}\x1b[0m`);
    } else if (packet.type === "Message") {
      console.log(`   📥 [Alice] Received Message from ${packet.senderId}: \x1b[32m'${packet.payload}'\x1b[0m`);
    } else if (packet.type === "ClientDisconnected") {
      console.log(`   🔌 [Alice] Received ClientDisconnected event from: \x1b[33m${packet.clientId}\x1b[0m`);
    }
  });

  bob.addEventListener("message", (event) => {
    const packet = decodePacket(event.data as ArrayBuffer);
    if (packet.type === "ClientConnected") {
      aliceId = packet.clientId || "";
      console.log(`   🦋 [Bob]   Received ClientConnected event -> Alice: \x1b[33m${aliceId}\x1b[0m`);
    } else if (packet.type === "Message") {
      console.log(`   📥 [Bob]   Received Message from ${packet.senderId}: \x1b[32m'${packet.payload}'\x1b[0m`);
    }
  });

  // Wait for initial sync propagation
  await new Promise((r) => setTimeout(r, 100));

  // 5. Message Exchange Tests
  console.log(`\n🌱 [4/5] Testing binary packet exchange:`);

  // Broadcast from Alice
  console.log(`   📤 [Alice -> Room] Broadcast: 'Hello room from TS'`);
  alice.send(encodeBroadcast("Hello room from TS"));
  await new Promise((r) => setTimeout(r, 100));

  // Unicast from Bob to Alice
  if (aliceId) {
    console.log(`   📤 [Bob -> Alice] Unicast: 'Private message TS'`);
    bob.send(encodeUnicast(aliceId, "Private message TS"));
    await new Promise((r) => setTimeout(r, 100));
  }

  // 6. Disconnect
  console.log(`\n🌱 [5/5] Disconnecting Bob...`);
  bob.close();
  await new Promise((r) => setTimeout(r, 100));
  alice.close();

  console.log(`\n\x1b[1;32m✨ TYPESCRIPT TEST CLIENT COMPLETED SUCCESSFULLY! ✨\x1b[0m\n`);
}

main().catch(console.error);
