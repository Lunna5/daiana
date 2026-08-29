import { describe, expect, it } from 'vitest';
import { RoomManager } from '../src/room/roomManager';

describe('RoomManager', () => {
  const peer1 = 'c1f7b889-4e78-4389-9407-73d8b28cf998';
  const peer2 = '550e8400-e29b-41d4-a716-446655440000';

  it('should track connected peers correctly', () => {
    const room = new RoomManager();
    expect(room.isEmpty()).toBe(true);
    expect(room.getPeerCount()).toBe(0);

    room.addPeer(peer1);
    expect(room.isEmpty()).toBe(false);
    expect(room.getPeerCount()).toBe(1);
    expect(room.hasPeer(peer1)).toBe(true);
    expect(room.hasPeer(peer2)).toBe(false);

    room.addPeer(peer2);
    expect(room.getPeerCount()).toBe(2);
    expect(room.getPeers()).toEqual([peer1, peer2]);

    expect(room.removePeer(peer1)).toBe(true);
    expect(room.hasPeer(peer1)).toBe(false);
    expect(room.getPeerCount()).toBe(1);

    expect(room.removePeer('non-existent')).toBe(false);

    room.clear();
    expect(room.isEmpty()).toBe(true);
    expect(room.getPeerCount()).toBe(0);
  });
});
