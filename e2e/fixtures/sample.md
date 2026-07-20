# Jarvis E2E Fixture

This document is seeded automatically when `JARVIS_E2E=1`.

The secret keyword for automated tests is **xyzzy-plugh**.

Jarvis indexes this file so WebDriver can ask a question and receive a Mock answer with citations.

## 中文段落（多字节回归用例）

Jarvis 是一个本地优先的个人知识中枢，支持对本地文档与飞书内容进行索引、混合检索与带引用的问答，并提供智能体模式以调用搜索、记忆与任务等工具；该段落刻意写得较长，使其切分后的文本块超过两百字节，用于在端到端层面覆盖引用摘要在多字节（中文）文本上的截断路径，防止再次出现字符边界 panic 导致界面一直停留在“思考中”的回归问题。
