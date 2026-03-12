// HTTP client for Connect RPC-style JSON API
const BASE_URL = 'http://localhost:3000';

export async function rpcCall<Req, Res>(service: string, method: string, body: Req): Promise<Res> {
  let res: Response;
  try {
    res = await fetch(`${BASE_URL}/${service}/${method}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
  } catch (err) {
    // Network error (backend not running, CORS, etc.)
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`${service}/${method} network error: ${message}`);
  }

  if (!res.ok) {
    let errorText: string;
    try {
      errorText = await res.text();
    } catch {
      errorText = res.statusText;
    }
    throw new Error(`${service}/${method} failed: ${res.status} ${errorText}`);
  }

  return res.json() as Promise<Res>;
}
