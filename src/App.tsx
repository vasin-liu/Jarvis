import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [count, setCount] = useState<number | null>(null);
  const [lastIndexed, setLastIndexed] = useState<string | null>(null);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    invoke<number>("source_count")
      .then(setCount)
      .catch((e) => setErr(String(e)));
  }, [lastIndexed]);

  async function handleIndexDemo() {
    setErr(null);
    try {
      const id = await invoke<string>("index_file", {
        path: "README.md",
      });
      setLastIndexed(id);
    } catch (e) {
      setErr(String(e));
    }
  }

  return (
    <main style={{ padding: 24, fontFamily: "system-ui" }}>
      <h1>知识中枢 · M2 索引管线</h1>
      {err ? <p>错误：{err}</p> : <p>来源数量：{count ?? "加载中…"}</p>}
      {lastIndexed && <p>最近索引：{lastIndexed}</p>}
      <button type="button" onClick={handleIndexDemo} style={{ marginTop: 12 }}>
        索引 README.md（冒烟）
      </button>
    </main>
  );
}

export default App;
