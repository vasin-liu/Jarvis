export interface Task {
  id: string;
  source_id?: string | null;
  source_title?: string | null;
  title: string;
  description?: string | null;
  status: "pending" | "done";
  created_at: number;
  updated_at: number;
}
