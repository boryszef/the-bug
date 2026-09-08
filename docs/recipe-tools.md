# Feature: recipe tools

## Why

A `Recipe` had one item list — everything it needed, all consumed on craft. That
can't express "you need an axe to carve this, but the axe isn't used up."
`STORY.md` §11 puts *tools* as their own tier in the progression
(materials → processing → **tools** → components → machines); the first one, the
Stone Axe, is already the target of the opening craft quest.

## Behaviour

A `Recipe` now has two lists:

- **`consumables: &'static [(Item, u32)]`** — spent when the recipe runs (this is
  the old `inputs`, renamed).
- **`tools: &'static [Item]`** — must be in the player's inventory to run the
  recipe. One unit of each is enough and **none are consumed**.

Both `Game::craft` (a known recipe) and `Game::experiment` (matching a recipe by
its consumables) enforce tools:

- **craft** checks consumables, then tools, then spends — so a missing tool
  leaves the consumables untouched. Logs
  `EventKind::CraftMissingTool { tool, output }` → "You need a Stone Axe to
  craft a Wooden Bow." (Polish: `tool` genitive, `output` accusative, like
  `CraftShortage`.) The player chose the recipe by name, so naming both is
  fine.
- **experiment** spends the combination first (every failed experiment does),
  then, if the consumables matched a recipe but a tool is missing, logs
  `EventKind::ExperimentMissingTool { items }` → "Experiment: 1 Branch ->
  you're missing a tool" **without** learning the recipe, producing the
  output, or naming the tool. The player is *discovering* — either name
  would give the recipe away (a 1-Branch combo that needs a Stone Axe is
  unmistakably the Arrow). This is a deliberate split from
  `CraftMissingTool`.

Same "reveal nothing" principle, positive side: while the player is building
an Experiment selection, `Player::experiment_would_discover(items)` (reusing
`find_matching`, ignoring tools) drives a green **"Looks good!"** hint when
the selection exactly matches an *undiscovered* recipe — a plain yes/no, no
name, no "how close". See `docs/gui-panels.md`.

### Craft panel

`viewmodel::crafting::CraftOption` carries `tools: Vec<CraftTool { item, present }>`
alongside `consumables`, and `enabled` requires every consumable met **and**
every tool present. The `gui` panel renders each tool in the recipe's row as a
bare item name — **no `have/need` number** — muted when held, in the error
colour when missing (`Wooden Bow  1/1 Branch  0/1 Cord  Stone Axe`). The frozen
`tui` panel appends them in brackets on the row text
(`Wooden Bow  (1/1 Branch, 0/1 Cord) [Stone Axe]`); it has no per-item colour, so
a missing tool only shows via the row being dimmed.

### What requires what

| Recipe | Consumables | Tools |
|---|---|---|
| Arrow | Branch ×1 | Stone Axe |
| Wooden Bow | Branch ×1, Cord ×1 | Stone Axe |

Every other recipe has `tools: &[]`.

## Scope

- `src/game/recipe.rs` — `Recipe.tools` field + `tools()` accessor; `tools: &[]`
  on every `RECIPES` entry except Arrow / Wooden Bow (`&[Item::StoneAxe]`).
- `src/game/player.rs` — `first_missing_tool(&[Item]) -> Option<Item>`.
- `src/game/event.rs` — `EventKind::CraftMissingTool { tool, output }` (craft)
  and `EventKind::ExperimentMissingTool { items }` (experiment, added later —
  the experiment message must not reveal the recipe).
- `src/game/mod.rs` — the tool check in `craft` and `experiment`.
- `src/i18n/mod.rs` + `locales/{en,pl}/main.ftl` — `event-craft-missing-tool`,
  `event-experiment-missing-tool`.
- `src/gui/mod.rs` — event-log colour (`CraftMissingTool` yellow with the
  craft family, `ExperimentMissingTool` cyan with the experiment family).
- `src/viewmodel/crafting.rs` — `CraftTool`, `CraftOption.tools`, `enabled`.
- `src/gui/craft.rs`, `src/tui/craft.rs` — rendering.

## Out of scope

- **Tool categories.** Tools are specific `Item`s. "Any axe" (stone / steel /…)
  or "anything sharp" (a `STORY.md` §9 property) would make an entry a
  `ToolReq { Exact(Item), Category(_) }` enum — a later change contained to
  `recipe.tools()`, `viewmodel::crafting`, and the two `first_missing_tool`
  call sites.
- **A required tool count** (`&[(Item, u32)]`) — one of each always suffices.
- **Tool wear / durability.** Tools are never consumed or degraded.
- **Tools gating world actions** (an axe to chop, a bow to hunt) — that's the
  separate "items become enhancers" epic in `TODO.md`.
- Any save-format change — recipes still persist as names only; `tools` comes
  from the const table on load.
