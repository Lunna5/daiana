/**
 * UUID byte utilities for zero-allocation parsing and encoding.
 */

const HEX_CHARS = Array.from({ length: 256 }, (_, i) => i.toString(16).padStart(2, '0'));
const HEX_REGEX = /^[0-9a-fA-F]{32}$/;

/**
 * Converts a 16-byte slice of a Uint8Array into a canonical UUID string (RFC 4122).
 *
 * @param bytes The byte array containing the 16-byte UUID
 * @param offset The starting index in the byte array (default: 0)
 * @returns The formatted UUID string (e.g. "550e8400-e29b-41d4-a716-446655440000")
 */
export function bytesToUuid(bytes: Uint8Array, offset: number = 0): string {
  if (bytes.length < offset + 16) {
    throw new Error(`Insufficient bytes for UUID: expected 16, got ${bytes.length - offset}`);
  }

  return (
    HEX_CHARS[bytes[offset]!]! +
    HEX_CHARS[bytes[offset + 1]!]! +
    HEX_CHARS[bytes[offset + 2]!]! +
    HEX_CHARS[bytes[offset + 3]!]! +
    '-' +
    HEX_CHARS[bytes[offset + 4]!]! +
    HEX_CHARS[bytes[offset + 5]!]! +
    '-' +
    HEX_CHARS[bytes[offset + 6]!]! +
    HEX_CHARS[bytes[offset + 7]!]! +
    '-' +
    HEX_CHARS[bytes[offset + 8]!]! +
    HEX_CHARS[bytes[offset + 9]!]! +
    '-' +
    HEX_CHARS[bytes[offset + 10]!]! +
    HEX_CHARS[bytes[offset + 11]!]! +
    HEX_CHARS[bytes[offset + 12]!]! +
    HEX_CHARS[bytes[offset + 13]!]! +
    HEX_CHARS[bytes[offset + 14]!]! +
    HEX_CHARS[bytes[offset + 15]!]!
  );
}

/**
 * Converts a canonical UUID string into a 16-byte Uint8Array.
 *
 * @param uuid The formatted UUID string (with or without hyphens)
 * @returns A new Uint8Array of exactly 16 bytes
 */
export function uuidToBytes(uuid: string): Uint8Array {
  const clean = uuid.replace(/-/g, '');
  if (clean.length !== 32 || !HEX_REGEX.test(clean)) {
    throw new Error(`Invalid UUID string: "${uuid}"`);
  }

  const bytes = new Uint8Array(16);
  for (let i = 0; i < 16; i++) {
    bytes[i] = parseInt(clean.substring(i * 2, i * 2 + 2), 16);
  }

  return bytes;
}

/**
 * Validates whether a given string is a valid UUID format.
 */
export function isValidUuid(uuid: string): boolean {
  const pattern = /^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/;
  return pattern.test(uuid);
}
