import { describe, expect, it } from "vitest";
import {
  formatIndexedAt,
  sourceKindLabel,
  statusLabel,
  statusTone,
} from "./sourceDisplay";

describe("sourceDisplay", () => {
  it("maps source kinds to labels", () => {
    expect(sourceKindLabel("local_file")).toBe("本地文件");
    expect(sourceKindLabel("lark_doc")).toBe("飞书文档");
    expect(sourceKindLabel("cursor_transcript")).toBe("Cursor 会话");
    expect(sourceKindLabel("memory")).toBe("记忆");
  });

  it("maps status to tone", () => {
    expect(statusTone("indexed")).toBe("ok");
    expect(statusTone("failed")).toBe("err");
    expect(statusLabel("pending")).toBe("待处理");
  });

  it("formats indexed timestamps", () => {
    expect(formatIndexedAt(null)).toBe("—");
    expect(formatIndexedAt(0)).toBe("—");
    expect(formatIndexedAt(1_700_000_000)).toContain("2023");
  });
});
