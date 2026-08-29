/**
 * Helper utilities for HTTP room creation and WebSocket URI derivation.
 */

/**
 * Sends a `POST /room/` request to the Daiana server to create a new room.
 *
 * @param httpBaseUrl The base HTTP/HTTPS URL of the Daiana server (e.g. `https://daiana.lunna.dev` or `http://localhost:8080`)
 * @param customFetch Optional custom `fetch` implementation (useful in older Node environments)
 * @returns A Promise resolving to the created room's UUID
 */
export async function createRoom(
  httpBaseUrl: string,
  customFetch: typeof fetch = globalThis.fetch
): Promise<string> {
  const base = httpBaseUrl.replace(/\/+$/, '');
  const endpoint = `${base}/room/`;

  const response = await customFetch(endpoint, {
    method: 'POST',
    headers: {
      Accept: 'application/json',
    },
  });

  if (!response.ok) {
    const errorText = await response.text().catch(() => '');
    throw new Error(`Failed to create room: HTTP ${response.status} ${response.statusText} ${errorText}`.trim());
  }

  const json = (await response.json()) as { id?: string; error?: string };

  if (json.error) {
    throw new Error(`Server returned error while creating room: ${json.error}`);
  }

  if (!json.id || typeof json.id !== 'string') {
    throw new Error(`Invalid server response: missing room "id" field in JSON`);
  }

  return json.id;
}

/**
 * Derives a WebSocket connection URL (`ws://` or `wss://`) from an HTTP base URL and room UUID.
 *
 * @param httpBaseUrl The base HTTP/HTTPS URL of the server
 * @param roomId The room UUID
 * @returns The full WebSocket URL pointing to `/room/{roomId}`
 */
export function deriveWebSocketUrl(httpBaseUrl: string, roomId: string): string {
  const base = httpBaseUrl.replace(/\/+$/, '');
  const wsBase = base.replace(/^http:\/\//i, 'ws://').replace(/^https:\/\//i, 'wss://');
  return `${wsBase}/room/${roomId}`;
}
