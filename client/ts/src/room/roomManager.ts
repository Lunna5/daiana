/**
 * Manages the set of connected peers within a room.
 */
export class RoomManager {
  private readonly peers: Set<string> = new Set();

  /**
   * Adds a connected peer UUID to the room.
   *
   * @param clientId The UUID of the connected peer
   */
  public addPeer(clientId: string): void {
    this.peers.add(clientId);
  }

  /**
   * Removes a disconnected peer UUID from the room.
   *
   * @param clientId The UUID of the disconnected peer
   * @returns `true` if the peer was in the room and removed, `false` otherwise
   */
  public removePeer(clientId: string): boolean {
    return this.peers.delete(clientId);
  }

  /**
   * Checks if a peer UUID is currently registered in the room.
   *
   * @param clientId The UUID of the peer to check
   */
  public hasPeer(clientId: string): boolean {
    return this.peers.has(clientId);
  }

  /**
   * Returns a snapshot array of all active peer UUIDs in the room.
   */
  public getPeers(): string[] {
    return Array.from(this.peers);
  }

  /**
   * Returns the total number of connected peers in the room.
   */
  public getPeerCount(): number {
    return this.peers.size;
  }

  /**
   * Checks if there are no other peers currently registered in the room.
   */
  public isEmpty(): boolean {
    return this.peers.size === 0;
  }

  /**
   * Clears all registered peers from the room.
   */
  public clear(): void {
    this.peers.clear();
  }
}
