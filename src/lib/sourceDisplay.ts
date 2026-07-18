export function sourceKindLabel(kind: string): string {
  switch (kind) {
    case "local_file":
      return "本地文件";
    case "lark_doc":
      return "飞书文档";
    case "lark_sheet":
      return "电子表格";
    case "lark_mail":
      return "邮件";
    case "lark_msg":
      return "会话";
    case "lark_file":
      return "云文件";
    case "cursor_transcript":
      return "Cursor 会话";
    case "memory":
      return "记忆";
    case "wiki_page":
      return "笔记页";
    default:
      return kind;
  }
}

export function statusLabel(status: string): string {
  switch (status) {
    case "indexed":
      return "已索引";
    case "failed":
      return "失败";
    case "pending":
      return "待处理";
    default:
      return status;
  }
}

export function statusTone(
  status: string,
): "ok" | "warn" | "err" | "muted" {
  switch (status) {
    case "indexed":
      return "ok";
    case "failed":
      return "err";
    case "pending":
      return "warn";
    default:
      return "muted";
  }
}

export function formatIndexedAt(ts?: number | null): string {
  if (ts == null || ts <= 0) return "—";
  return new Date(ts * 1000).toLocaleString();
}
