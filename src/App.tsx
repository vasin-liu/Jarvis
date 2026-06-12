import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface Citation {
  chunk_id: number;
  source_id: string;
  source_title: string;
  source_uri: string;
  loc: string;
  excerpt: string;
}

interface AskResponse {
  answer: string;
  citations: Citation[];
}

function App() {
  const [count, setCount] = useState<number | null>(null);
  const [lastIndexed, setLastIndexed] = useState<string | null>(null);
  const [question, setQuestion] = useState("");
  const [answer, setAnswer] = useState<AskResponse | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    invoke<number>("source_count")
      .then(setCount)
      .catch((e) => setErr(String(e)));
  }, [lastIndexed, answer]);

  async function handlePickAndIndex() {
    setErr(null);
    setBusy(true);
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Documents",
            extensions: ["txt", "md", "markdown"],
          },
        ],
      });

      if (selected === null) {
        return;
      }

      const id = await invoke<string>("index_file", { path: selected });
      setLastIndexed(id);
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleAsk() {
    const q = question.trim();
    if (!q) {
      return;
    }
    setErr(null);
    setBusy(true);
    try {
      const resp = await invoke<AskResponse>("ask_question", { question: q });
      setAnswer(resp);
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main style={{ padding: 24, fontFamily: "system-ui", maxWidth: 720 }}>
      <h1>知识中枢 · M3 问答</h1>
      {err ? <p>错误：{err}</p> : <p>来源数量：{count ?? "加载中…"}</p>}
      {lastIndexed && <p>最近索引：{lastIndexed}</p>}

      <section style={{ marginTop: 16 }}>
        <button type="button" onClick={handlePickAndIndex} disabled={busy}>
          {busy ? "处理中…" : "选择文件并索引"}
        </button>
      </section>

      <section style={{ marginTop: 24 }}>
        <textarea
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          placeholder="输入问题…"
          rows={3}
          style={{ width: "100%", padding: 8, fontSize: 14 }}
        />
        <button
          type="button"
          onClick={handleAsk}
          disabled={busy || !question.trim()}
          style={{ marginTop: 8 }}
        >
          提问
        </button>
      </section>

      {answer && (
        <section style={{ marginTop: 24 }}>
          <h2>回答</h2>
          <p>{answer.answer}</p>
          {answer.citations.length > 0 && (
            <>
              <h3>引用</h3>
              <ul style={{ paddingLeft: 18 }}>
                {answer.citations.map((c) => (
                  <li key={c.chunk_id} style={{ marginBottom: 12 }}>
                    <strong>{c.source_title}</strong> · {c.loc}
                    <div style={{ fontSize: 12, color: "#555" }}>{c.source_uri}</div>
                    <div style={{ fontSize: 13, marginTop: 4 }}>{c.excerpt}</div>
                  </li>
                ))}
              </ul>
            </>
          )}
        </section>
      )}
    </main>
  );
}

export default App;
