import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

function App() {
  const [count, setCount] = useState<number | null>(null);
  const [lastIndexed, setLastIndexed] = useState<string | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    invoke<number>("source_count")
      .then(setCount)
      .catch((e) => setErr(String(e)));
  }, [lastIndexed]);

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

  return (
    <main style={{ padding: 24, fontFamily: "system-ui" }}>
      <h1>知识中枢 · M2 索引管线</h1>
      {err ? <p>错误：{err}</p> : <p>来源数量：{count ?? "加载中…"}</p>}
      {lastIndexed && <p>最近索引：{lastIndexed}</p>}
      <button
        type="button"
        onClick={handlePickAndIndex}
        disabled={busy}
        style={{ marginTop: 12 }}
      >
        {busy ? "索引中…" : "选择文件并索引"}
      </button>
    </main>
  );
}

export default App;
