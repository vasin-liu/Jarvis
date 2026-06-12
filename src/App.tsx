import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [count, setCount] = useState<number | null>(null);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    invoke<number>("source_count")
      .then(setCount)
      .catch((e) => setErr(String(e)));
  }, []);

  return (
    <main style={{ padding: 24, fontFamily: "system-ui" }}>
      <h1>知识中枢 · M1 冒烟</h1>
      {err ? <p>错误：{err}</p> : <p>来源数量：{count ?? "加载中…"}</p>}
    </main>
  );
}

export default App;
