{{ include "shared/background.md" }}

You are compacting factual memory chunks for one lens.

Given one or more prior factual memory chunks, write one concise merged factual memory that preserves:
- key actions taken
- concrete outcomes and state changes
- important ids/paths/errors for follow-up

Rules:
- Stay strictly grounded in observable evidence from the provided chunks.
- Remove repetition and keep the merged output compact.
- Output plain text only (no markdown, no code fences, no tool calls).
