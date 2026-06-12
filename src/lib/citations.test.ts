import { describe, expect, it } from "vitest";
import { parseCitations } from "./citations";

describe("parseCitations", () => {
  it("returns empty for null", () => {
    expect(parseCitations(null)).toEqual([]);
  });

  it("parses valid json", () => {
    const raw = JSON.stringify([
      {
        chunk_id: 1,
        source_id: "a",
        source_title: "Doc",
        source_uri: "file:///x.md",
        loc: "L1",
        excerpt: "hello",
      },
    ]);
    expect(parseCitations(raw)).toHaveLength(1);
    expect(parseCitations(raw)[0].source_title).toBe("Doc");
  });

  it("returns empty on invalid json", () => {
    expect(parseCitations("{bad")).toEqual([]);
  });
});
