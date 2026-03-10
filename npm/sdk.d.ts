export function init(apiKey: string): void;

export interface HttpbinClient {
  getUuid(): Promise<string>;
}

export const httpbin: HttpbinClient | null;
