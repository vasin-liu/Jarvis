export interface Citation {
  chunk_id: number;
  source_id: string;
  source_title: string;
  source_uri: string;
  loc: string;
  excerpt: string;
}

export function parseCitations(raw?: string | null): Citation[] {
  if (!raw) return [];
  try {
    return JSON.parse(raw) as Citation[];
  } catch {
    return [];
  }
}
