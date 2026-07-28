export interface Source {
  id: string;
  kind: string;
  uri: string;
  title: string;
  status: string;
  indexed_at?: number | null;
  error?: string | null;
  summary?: string | null;
}

export interface CursorTranscriptSummary {
  session_id: string;
  project: string;
  path: string;
  uri: string;
}

export interface InsightsReport {
  summarized: number;
  tasksExtracted: number;
  failed: number;
}

export interface RelatedSource {
  sourceId: string;
  title: string;
  kind: string;
  snippet: string;
}

export interface RebuildReport {
  indexed: number;
  failed: number;
  skipped: number;
}
