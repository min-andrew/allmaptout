const BASE_URL =
  import.meta.env.PUBLIC_API_URL ||
  (import.meta.env.DEV ? "http://localhost:3001" : "/api");

type RequestConfig = {
  method: string;
  url: string;
  data?: unknown;
  params?: Record<string, string>;
  headers?: Record<string, string>;
};

export class ApiError extends Error {
  constructor(
    public status: number,
    public statusText: string,
    public body?: { error?: string },
  ) {
    super(body?.error || `${status} ${statusText}`);
    this.name = "ApiError";
  }

  get isNotFound(): boolean {
    return this.status === 404;
  }

  get isUnauthorized(): boolean {
    return this.status === 401;
  }

  get isServerError(): boolean {
    return this.status >= 500;
  }

  get userMessage(): string {
    if (this.isServerError) {
      return "Something went wrong. Please try again later.";
    }
    return this.body?.error || this.statusText;
  }
}

export class NetworkError extends Error {
  constructor(cause?: Error) {
    super(
      "Unable to connect to the server. Please check your internet connection.",
    );
    this.name = "NetworkError";
    this.cause = cause;
  }
}

export async function client<T>(config: RequestConfig): Promise<T> {
  const url = new URL(config.url, BASE_URL);

  if (config.params) {
    Object.entries(config.params).forEach(([key, value]) => {
      url.searchParams.set(key, value);
    });
  }

  let response: Response;
  try {
    response = await fetch(url.toString(), {
      method: config.method,
      headers: {
        "Content-Type": "application/json",
        ...config.headers,
      },
      body: config.data ? JSON.stringify(config.data) : undefined,
    });
  } catch (err) {
    throw new NetworkError(err instanceof Error ? err : undefined);
  }

  if (!response.ok) {
    let body: { error?: string } | undefined;
    try {
      body = await response.json();
    } catch {
      // Response body is not JSON
    }
    throw new ApiError(response.status, response.statusText, body);
  }

  return response.json();
}
