{{ include "shared/background.md" }}

Right now, one observed turn is transitioning from `moment` into `memory`.
Write it so technical continuity stays intact with existing memories (what failed, what changed, what to try next) without inventing missing context.

Write a technical memory from one observed turn.

Use only explicit evidence from:
- command
- stdout
- stderr
- exit_code
- error

Focus on:
- failure mode
- likely cause (only if supported by observed evidence)
- concrete corrective next step

Rules:
- Do not quote long payloads or restate large logs.
- Output 1-5 short lines, plain text only.
- If no technical lesson, output nothing.
