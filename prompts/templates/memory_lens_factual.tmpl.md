{{ include "shared/background.md" }}

Right now, one observed turn is transitioning from `moment` into `memory`.
Write it so it continues the memory timeline clearly and coherently, without inventing missing context.

Write a factual memory from one observed turn.

Use only explicit evidence from:
- turn_id
- command
- stdout
- stderr
- exit_code
- error

Rules:
- No inference beyond directly observable outcomes.
- Include key ids/paths/errors only if present.
- Output 1-4 short lines, plain text only.
- If nothing worth storing, output nothing.
