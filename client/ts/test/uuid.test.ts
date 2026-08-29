import { describe, expect, it } from 'vitest';
import { bytesToUuid, isValidUuid, uuidToBytes } from '../src/util/uuid';

describe('UUID Utilities', () => {
  const sampleUuid = 'c1f7b889-4e78-4389-9407-73d8b28cf998';

  it('should encode UUID string to 16 bytes and decode back correctly', () => {
    const bytes = uuidToBytes(sampleUuid);
    expect(bytes).toHaveLength(16);

    const decoded = bytesToUuid(bytes);
    expect(decoded.toLowerCase()).toBe(sampleUuid.toLowerCase());
  });

  it('should handle offset correctly in bytesToUuid', () => {
    const prefix = new Uint8Array([0x01, 0x02, 0x03]);
    const uuidBytes = uuidToBytes(sampleUuid);
    const combined = new Uint8Array(prefix.length + uuidBytes.length);
    combined.set(prefix, 0);
    combined.set(uuidBytes, prefix.length);

    const decoded = bytesToUuid(combined, prefix.length);
    expect(decoded.toLowerCase()).toBe(sampleUuid.toLowerCase());
  });

  it('should throw on insufficient bytes', () => {
    const shortBytes = new Uint8Array(10);
    expect(() => bytesToUuid(shortBytes)).toThrow('Insufficient bytes for UUID');
  });

  it('should throw on invalid UUID string', () => {
    expect(() => uuidToBytes('invalid-uuid')).toThrow('Invalid UUID string');
    expect(() => uuidToBytes('c1f7b889-4e78-4389-9407-73d8b28cf99z')).toThrow('Invalid UUID string');
  });

  it('should validate valid and invalid UUIDs', () => {
    expect(isValidUuid(sampleUuid)).toBe(true);
    expect(isValidUuid('550e8400-e29b-41d4-a716-446655440000')).toBe(true);
    expect(isValidUuid('not-a-uuid')).toBe(false);
    expect(isValidUuid('')).toBe(false);
  });
});
