# the-bug

A terminal crafting/exploration game (Rust, ratatui) set in a post-apocalyptic
future reshaped by alien-derived biological computing.

Before making structural changes — new modules, moving logic between
`game`/`viewmodel`/`ui`, changing visibility, touching persistence — read
[ARCHITECTURE.md](ARCHITECTURE.md). It's the standing record of *why* the
codebase is shaped the way it is, not a specific feature's rationale.

Before writing quest content, flavor text, or anything narrative-facing,
read [STORY.md](STORY.md). The world's backstory is meant to unfold in
layers through play — don't reveal more in a quest than that quest should.

`docs/*.md` — one file per feature or decision: why it was built, what
changed, what was deliberately left out of scope. Check for one relevant to
what you're touching before assuming; write one for anything more than a
small fix (see ARCHITECTURE.md's Workflow section for the full process).

`TODO.md` — the backlog (`## TODO` / `## DONE`). Check it before starting
unprompted work; move an item to `## DONE` when it ships.
