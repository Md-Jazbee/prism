# Baseline measurement runbook (explore without Prism)

1. Check out the pinned SHA from `fixtures/repos/<repo>.md`.
2. Give the agent **only** filesystem tools: Read, Grep, Glob, Bash (no MCP graph tools).
3. Run each gold task; capture transcript.
4. Count approximate tokens (provider usage or tiktoken estimate) and tool calls.
5. Store JSON under `eval/baselines/<repo>/<task_id>.json` with keys:
   `task_id`, `protocol`, `tokens_in`, `tokens_out`, `tool_calls`, `latency_ms`.

Offline-first: structural hop counts and tool-call counts are valid before LLM judges exist.
