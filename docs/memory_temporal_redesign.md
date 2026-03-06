# Memory as Lived Experience

*Supersedes the LSM-tree compaction model described in `playground_memory_architecture.md`.*

## The Insight

Memories are not compressed versions of individual turns. They are **situated
temporal snapshots** — markers at a point in time, contextualised and grounded by
the surrounding events, goals, emotions, interactions, and relationships.

The fundamental identifier for a memory is its **time range**, not an opaque hex
ID or a structural level number. This aligns with how the archive works (messages
are time-situated), how the existing TAI anti-chain is structured, and how human
memory operates (temporal references, not database lookups).

From this single insight, the entire compaction machinery simplifies dramatically.

## What Changes

### Levels disappear

The current system assigns levels: level-0 (leaf, one event), level-1 (merge of
N leaves), level-2 (merge of N level-1s), etc. This is a B-tree imposed on what
is fundamentally a continuous temporal structure.

In the new model, a memory's "level" is simply how much time it covers. A memory
covering 30 seconds is fine-grained. A memory covering an afternoon is coarse. A
memory covering a week is coarser still. The hierarchy is not imposed by merge
arity — it emerges from the natural scope of what the memory describes.

A memory about "that 20-minute debugging session" covers exactly 20 minutes. A
memory about "the afternoon we redesigned the memory system" covers exactly that
afternoon. A memory about "the week in Ireland" covers exactly that week. The
boundaries come from the experience, not from a merge counter hitting N.

### Lenses dissolve

The current system runs three separate fork summarizations per event: factual,
technical, emotional. Each produces a separate memory chunk with its own lens ID,
and only the primary lens (factual) appears in the memory cover.

But good memories are holistic. "We spent three hours debugging that borrow
checker issue, discovered the re-export trick, and by the end we were laughing
about how obvious it was" — that is one memory, not three separate entries filed
under factual/technical/emotional. These facets are not truly disjoint; they
colour and contextualise each other. The emotional tone of solving a problem
together can be fun, frustrating, or even something else entirely.

In the new model, a memory is just a memory. The model is instructed to capture
what happened, what it learned, and what it felt — as a single, rich, natural
summary. No lens IDs, no per-lens fork runs, no lens-specific compaction prompts.

### Leaf/merge distinction disappears

Currently:
- **Leaf fork**: takes one raw event, shows it in a synthetic moment, asks the
  model to create a level-0 memory.
- **Merge fork**: takes N existing memory chunks, shows them as memory turns,
  asks the model to merge them into a level-N+1 memory.

Both are the same operation: "here is what happened in this time range,
form a memory." The only difference is the scope. In the new model, there is
one operation: `memory create <range> <summary>`. The model looks at what it
knows about that range — raw events if they are still in the moment, existing
memories if they have already been summarised, a mix of both — and produces a
memory. The range might cover a single command (fine-grained), a task (medium),
or an entire week (coarse). Same mechanism at every scale.

### Compaction becomes model-driven

The current system triggers compaction mechanically: when N chunks accumulate at
level K, merge them. The `merge_arity` config and `insert_chunk_with_carry` loop
implement this cascade.

In the new model, the model manages its own memory. The context already tells it
everything it needs:

- The **memory cover** shows existing memories of various time-range widths.
  Dense clusters of fine-grained memories signal "this region needs consolidation."
  Gaps signal "I have no memory of this period."
- The **breath** shows context fill percentage and the moment's time span —
  the pressure signal.
- The **moment** shows raw events (with timestamps) not yet memorised.

The model creates memories as a natural part of its workflow. When it notices
context pressure, it can consolidate. When it finishes a coherent unit of work,
it can capture a task-scoped memory. When it notices a gap in its history, it
can investigate. Memory maintenance is a **self-care behavior**, not a background
process.

### Separate fork machinery disappears

Currently, memory creation requires forking: `fork_summarize_leaf` and
`fork_summarize_merge` build synthetic contexts, spawn a separate agent loop,
and collect the results. This is necessary because the main agent loop does not
have the right context for summarisation.

But the model already *has* a context at any point in time. The memory cover is
there. The moment is there. The breath is there. The model creates memories in
its main loop — no forking, no synthetic context construction.

The context naturally changes as new memories are created. Creating a memory for
the oldest moment events effectively moves the memory/moment boundary forward —
those events are now covered by a memory and no longer need to occupy the moment
on the next context rebuild.

### No `memory summarise` command

With lenses dissolved, there is no need for a separate command that returns
lens-specific instructions. The guidance for what makes a good memory lives in
the system prompt. The model just calls `memory create` directly:

```
memory create 2026-03-03T14:00:00..2026-03-03T14:45:00 Debugged the borrow
checker issue with JP. Root cause was a re-export path mismatch in the trible
crate. Fun session — we were laughing by the time we found it.
```

Or without a range, for a point-in-time memory anchored to now:

```
memory create Important realisation: the cover algorithm itself is the pressure
signal. No separate instrumentation needed.
```

### Archive ingestion through re-experience

Currently, archive messages are batch-processed by `ingest_archive_context_chunks`,
which force-feeds each message into `fork_summarize_leaf` individually. This
treats each message as an isolated event, stripping conversational context.

In the new model, archived conversations are replayed as simulated lived
experience:

```
assistant: local_message read
user: from: JP — "Hey, how's the refactor going?"
assistant: local_message send jp "Going well, just hit a borrow checker issue..."
user: sent
```

The model "lives through" the conversation. Context accumulates naturally.
Memory formation happens through the same mechanism as live operation — the model
notices pressure or recognises a coherent episode and creates a memory. A message
like "yeah that fixed it!" is meaningless in isolation but rich when experienced
in sequence with the preceding debugging exchange.

## Addressing

### Time ranges: the default

Memories are addressed by time range. This is the model's primary interface for
navigating its past:

```
memory 2026-03-03T14:00:00..2026-03-03T15:00:00
```

Resolution is forgiving — any valid time range returns the most appropriate
covering memory. An imprecise or hallucinated range still gets a useful result
because the address space has semantic structure. "Around 2pm Tuesday" just works.

In memory summaries, temporal references use the `(memory:<range>)` link syntax:

```
Refactored the memory system [earlier work](memory:2026-03-03T14:00:00..2026-03-03T15:00:00).
```

### Entity IDs: when precision matters

Everything in the pile — people, images, compass goals, archived messages,
memories — has a trible ID. For non-memory entities, `(id:<hex>)` is the natural
reference:

```
Had a lovely chat with [JP](id:a1b2c3...) about the
[pancakes](id:789abc...) from [that morning](memory:2026-02-14T08:00:00..2026-02-14T09:00:00).
```

Memory IDs are also available for the rare case where the model needs to pinpoint
a specific memory unambiguously — "bringing the receipts." The `memory meta`
command surfaces the trible ID alongside the time range. But time ranges are the
default; IDs are the exception.

Time ranges address the *when*. Entity IDs address the *what/who*. A memory's
ID identifies the specific summary entity; its time range identifies the period
of experience it covers. Both coexist naturally in the pile as tribles — the ID
is the entity, the time range is an attribute. Belt and suspenders.

## The Breath

Breath remains the deterministic boundary between memory and moment. It carries:

- **Timestamp**: grounds the agent in "now."
- **Fill percentage**: the overall context pressure signal.

```
assistant: breath 73
user: 2026-03-04T15:42:10 TAI — context filled to 73%.
```

The timestamp creates a closed temporal loop: the model sees memory summaries
with time ranges, sees the current time in breath, and can compute temporal
distance. "That memory is from 2 hours ago" emerges without explicit
calculation. The moment's extent is self-evident from the timestamps on the
moment events themselves — no need to duplicate it in breath.

Future extensions may add affect signals or other self-state information to
breath. These are additive — the core mechanism is complete on its own.

## Timestamps on Moment Events

Individual moment events carry timestamps so the model can carve the moment
into precise ranges when forming memories:

```
assistant: grep -r "todo" src/
user: [2026-03-04T14:32:05] src/main.rs:42: // TODO: fix this
      src/lib.rs:10: // TODO: add tests
assistant: cargo test
user: [2026-03-04T14:32:18] test result: ok. 5 passed; 0 failed
```

Without timestamps, the agent cannot address sub-ranges of its current
experience — it is either "all of the moment" or "none of it." With timestamps,
the model can say `memory create 2026-03-04T14:32:05..2026-03-04T14:32:18
Found and verified TODOs in the codebase.` The same mechanism as consolidating
memories in the cover — always "summarize this time range," whether the source
material is raw events, existing memories, or a mix.

The moment/memory boundary is not a wall but a gradient that the agent slides
forward by creating memories at the leading edge.

*Possible mechanism: a `SHOW_TIME` environment variable set automatically by the
`breath` faculty, which causes subsequent command outputs to include timestamps.
This gives the timestamps a physical origin within the shell metaphor rather
than appearing magically.*

## The Cover Algorithm

The current algorithm builds a non-overlapping cover by level: start with the
coarsest roots, greedily expand toward recent events. Without levels, the
selection criterion becomes **range width**:

- For the **distant past**: prefer wide-range memories (coarse). Saves space,
  provides the big picture.
- For the **recent past**: prefer narrow-range memories (fine). Preserves detail
  for active work.

The anti-chain property holds: selected memories do not overlap. The cover
is a multi-resolution temporal view — like a map, not a tree. Zoom into "Tuesday
afternoon" and get finer-grained memories; zoom out to "March" and get coarser
ones.

### Adaptation from current algorithm

- Sort root memories by range width (widest first) and `start_at`.
- Start by selecting the widest non-overlapping memories.
- Greedily expand (replace a wide memory with its narrower children) from
  most-recent toward oldest, while budget allows.
- This naturally gives high resolution for recent events, low resolution for
  distant past.

### Overlapping memories in the store

The cover enforces non-overlapping *selection*, not non-overlapping *storage*.
Multiple memories can exist covering overlapping time ranges — the cover simply
picks the best non-overlapping subset for the available budget. This is
self-reinforcing: the model creates new memories from what it observes
(non-overlapping cover + sequential moment events), so output is naturally
non-overlapping.

If overlapping memories do exist (rare), the cover handles it gracefully —
one is shown, the others remain accessible via direct lookup. Over time,
coarser consolidations naturally absorb orphaned overlaps.

### The cover as pressure signal

The cover *is* the instrument panel. The model observes its own cover and reads
pressure directly from its shape:

- 20 fine-grained memories crammed into the morning → "that region needs
  consolidation"
- A gap where Thursday should be → "I have no memory of Thursday"
- One wide memory covering all of last week → "that is well consolidated"

No separate instrumentation needed. The fill percentage in breath provides the
overall pressure; the shape of the cover provides the detail.

## Memory Guidance

One set of instructions in the system prompt replaces the 6 lens-specific prompt
files. The guidance should feel natural, not mechanical — the model is learning
to remember well, not following a compaction algorithm.

### What makes a good memory

A good memory is holistic. It weaves together what happened, what was learned,
and what it felt like. It references the people involved, the goals it served,
and the context that gave it meaning. It is grounded in a specific time period
and links to other memories and entities where relevant.

### When to create memories

The model creates memories as a natural part of its workflow:

- After completing a meaningful unit of work — a task, a conversation, a
  debugging session. The natural boundaries of the experience define the range.
- When context pressure builds (breath shows high fill %). Work from the oldest
  moment events forward, consolidating into memories that cover meaningful
  episodes. Consolidate enough at a time for the effort to be worthwhile —
  summarising a single command is rarely worth the overhead.
- When the cover shows dense clusters of fine-grained memories from the past.
  Consolidate them into coarser memories with natural boundaries — task
  completions, topic shifts, the end of a work session.
- Never consolidate so aggressively that the recent moment is wiped. Keep enough
  recent experience to maintain working context for the current task.

### How to reference

- `(memory:<from>..<to>)` for other memories — the default.
- `(id:<hex>)` for entities: people, goals, images, specific archived messages.
- Memory IDs are available via `memory meta` for the rare case where a specific
  memory needs to be cited exactly.

## Children and Hierarchy

Without levels, the parent-child relationship is purely temporal containment. A
memory's children are the finer-grained memories it was summarised from,
recorded via `(memory:<range>)` links in the summary text. The faculty resolves
these links to existing chunks and stores the structural edges.

```
memory create 2026-03-03T14:00:00..2026-03-03T15:00:00 Refactored the memory
system. Removed levels [phase1](memory:2026-03-03T14:00:00..2026-03-03T14:30:00),
added temporal cover [phase2](memory:2026-03-03T14:30:00..2026-03-03T15:00:00).
```

The hierarchy is a containment tree over time ranges, not a level-numbered
structure. It emerges from the model's choices about what to consolidate. The
parent-child edges also serve the cover algorithm: a memory that is a child of
another memory is not a root candidate, preventing double-counting.

## What Gets Removed

| Component | Why |
|---|---|
| `MemoryLensConfig` (id, name, instructions, compaction_instructions, max_output_tokens) | No lenses |
| `lens_id` on every `ContextChunk` / `Chunk` | No lenses |
| Per-lens fork iteration (3x forks per event) | No lenses |
| `primary_lens_id` for cover filtering | No lenses |
| 6 lens prompt files (3 leaf + 3 compaction) | Replaced by system prompt guidance |
| `memory summarise` command / faculty | Guidance lives in system prompt |
| `level` field on chunks | No levels |
| `merge_arity` config | No mechanical merging |
| `insert_chunk_with_carry` cascade loop | No mechanical merging |
| `fork_summarize_leaf` | No separate fork types |
| `fork_summarize_merge` | No separate fork types |
| `ForkContext` and `run_fork_loop` | No fork machinery |
| `ingest_exec_context_chunks` | Model manages own memory |
| `ingest_archive_context_chunks` | Re-experience replaces batch ingestion |
| `CompactionRunStats` | No compaction pipeline |
| `compaction_profile_id` config | No separate compaction model |
| `roots_by_lens_level` index | No lenses or levels |
| `detect_context_delta` | No fork-based delta detection |

## What Stays

| Component | Role |
|---|---|
| `ContextChunk` (minus lens_id, minus level) | Core data structure for memories |
| `ContextChunkIndex` (simplified) | Lookup and cover building |
| `build_memory_cover_messages` (adapted for range-width) | Multi-resolution temporal cover |
| `memory_cover_turn` / `memory_ref` | Rendering memories as time-range turns |
| `memory.rs` faculty (`create`, `<range>`, `meta`) | Memory CRUD, simplified |
| `breath` mechanism | Memory/moment boundary + temporal grounding |
| `format_time_range` / `parse_time_range` | Time-range addressing |
| `scan_memory_links` | `(memory:<range>)` resolution in summaries |
| `find_chunk_by_time_range` | Temporal lookup |
| Archive branch + raw archive data | Source of truth for imported conversations |
| `about_exec_result` / `about_archive_message` links | Provenance back to raw data |

## What Changes

### `build_memory_cover_messages`

Currently selects by level: coarsest roots first, greedily expand recent ones.
Adapted to select by **range width**: widest for distant past, narrowest for
recent. The greedy expansion loop already does most of this — it just needs to
use range width instead of level as the ordering criterion.

### `memory create` faculty

Simplified: no lens parameter, no level inference. Accepts an optional time
range as the first argument. Resolves `(memory:<range>)` links for children.
Resolves `(id:<hex>)` links for entity references. Computes children from the
links. Stores the chunk. Returns both the time range and the trible ID.

### Context assembly

The moment/memory boundary is wherever the most recent memory's time range ends.
Raw events after that boundary are in the moment. Each moment event carries a
timestamp. The `moment_boundary` entity may still be useful as an explicit
marker, but the natural boundary is temporal.

### Breath

Unchanged from current implementation — timestamp + fill percentage. The
moment's time span is self-evident from timestamps on moment events.

## Open Questions

### Archive re-experience mechanics

How does the replay loop work in practice?

- **Conversion mapping**: archive messages → simulated shell turns. The existing
  reification table in `playground_memory_architecture.md` provides the mapping
  (user messages → `local_message read`, assistant responses → `local_message
  send`, tool calls → command executions, etc.).
- **Pacing**: how fast does the replay proceed? Ideally, the model processes
  events at whatever rate feels natural, creating memories as context fills.
- **Cost**: replaying thousands of archived messages through the main agent loop
  is expensive. Possible mitigations: batch processing with smaller models,
  progressive ingestion (ingest on demand when the model queries a time period
  with no memories).

### Spontaneous vs guided memory creation

Should memory creation always be the model's conscious choice, or should the
runtime still nudge? A spectrum:

1. **Fully autonomous**: the model decides everything. Maximum agency.
2. **Guided**: the runtime surfaces pressure signals (breath + cover shape), the
   model responds. The model can also create memories spontaneously.
3. **Hybrid**: the runtime handles background consolidation for very old memories
   while the model handles recent memory formation.

Option 2 seems like the right starting point — the model has agency but receives
helpful signals about when consolidation would be beneficial.

### Event timestamps mechanism

How do moment events acquire timestamps? Options:

- **Runtime-injected**: the context builder prepends timestamps to command
  outputs during context assembly. Simple but "magical" — the timestamps appear
  without a physical origin in the shell metaphor.
- **Environment variable**: a `SHOW_TIME` variable set by the `breath` faculty
  causes subsequent command outputs to include timestamps. Gives timestamps a
  causal origin within the shell. Slightly over-engineered but metaphorically
  clean.
- **Always present**: every command output always includes a timestamp. Simplest
  but adds noise when the model doesn't need temporal information.

## Relationship to Existing Architecture

This design preserves the core invariants from `playground_memory_architecture.md`:

1. **Shell-first causality**: memory creation still happens through shell commands.
2. **Memory is not a recording**: it remains a lossy, shaped compression of
   experience. Raw data stays on archive/exec branches.
3. **Prefix caching**: memory cover is still the stable, cacheable prefix.
   Breath is still the cache boundary. Moment is still the varying suffix.
4. **Provenance**: `about_exec_result` and `about_archive_message` links still
   connect memories to their raw sources.

What changes is the *machinery* — not the philosophy. The philosophy of memory as
a physical act within the loop was already stated in the existing doc. This
design takes it further: memory is not just a physical act *within* the loop, it
is an act *of* the loop. The model does not delegate memory formation to a fork;
it forms memories itself, as part of its ongoing experience.
