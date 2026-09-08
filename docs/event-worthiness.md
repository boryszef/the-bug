# Decision: what merits an event

## Why

`TODO.md` carried a "review events" note before this was written down:
"event should reflect important messages coming from the game, not just
reflect user actions." The village-crafting feature
(`docs/village-crafting.md`) put that into practice for the first time —
craft/experiment/disassemble away from the Village do nothing and log
nothing — which surfaced the need to state the rule plainly enough to apply
consistently, rather than re-litigating it per feature. This is that rule,
plus a second one: a random roll merits a record even when it comes up
empty — *if* the attempt cost the player something regardless of how the
roll came out.

Nothing in `src/` changes here. Several existing `EventKind` variants don't
comply (marked **candidate** in the table below) — they're deliberately left
alone, folded into the "review events" TODO item as concrete follow-up work
rather than done piecemeal now.

## The rule

Log an event when **either** holds; if neither does, log nothing.

1. **Player-visible state actually changed.** Inventory, known recipes, XP,
   quest state — something the player can see or track was different after
   the action than before it. This is true even when the change was fully
   expected (a successful craft is no surprise to the player who clicked
   Craft) — it's the confirming record of what the game actually did, not
   new information for its own sake. Its converse is the important part: an
   action that changes **nothing** — a refused craft, a blocked hunt — isn't
   a result at all, just an echo of the click that produced it, so there's
   nothing to report.
2. **The attempt had a real cost, and its outcome was random.** A resource
   spent, or some other consequence the player pays regardless of how the
   roll comes out — if the roll then comes up empty, the cost still
   happened and deserves confirming (hunting always spends an arrow; a miss
   still burns it, so `HuntMissed` is worth a line even though nothing was
   gained). The reverse matters just as much: a **costless, freely-repeatable**
   roll doesn't merit a line on a miss, no matter how random. Searching
   costs nothing beyond a temporary, self-recovering dip in that one tile's
   own odds (`adjust_probability`'s decay window) — the player can search
   again immediately, for free, as many times as they like. A miss there
   isn't a loss; it's nothing, and "nothing" needs no announcement when
   trying again is free.

A rough test for rule 1: could the player have predicted the exact resulting
state before acting, with no game state hidden from them? If yes and it
turned out that way, it's not really "information from the game" — it's
just their own decision replayed back. Rule 2 exists for the one case that
test would wrongly silence: an attempt that cost something no matter the
outcome. Randomness alone isn't the qualifier — a free, repeatable roll
resolving to nothing is exactly as uninformative as a deterministic refusal,
and stays silent for the same reason.

## Applying it to every `EventKind` today

| `EventKind` (trigger) | State changed? | Random roll? | Verdict |
|---|---|---|---|
| `Awoke` | — | no | exception — the session's opening beat, not the result of any action |
| `Found` (search) | yes (item gained) | yes | keep |
| `QuestAccepted` | yes (quest becomes active) | no | keep — rule 1 |
| `QuestCompleted` | yes (reward granted) | no | keep — rule 1 |
| `Crafted` | yes | no | keep — rule 1 |
| `Experimented` | yes | no | keep — rule 1 |
| `ExperimentFailed` | yes (items spent, lost) | no | keep — rule 1 |
| `Disassembled` | yes | no | keep — rule 1 |
| `Hunted` | yes | yes | keep |
| `HuntMissed` | yes (arrow already spent — `hunt` spends it *before* rolling) | yes, costly | keep — rule 1 and rule 2 agree |
| `ExperimentMissingTool` | yes (items already spent by the time the tool check runs) | no | keep — rule 1 |
| `UnknownRecipe` | no | no | **candidate** — a pure refusal, same shape as the village check |
| `CraftShortage` | no (checked before spending) | no | **candidate** |
| `CraftMissingTool` (from `craft`) | no (checked before spending) | no | **candidate** |
| `ExperimentShortage` | no (checked before spending) | no | **candidate** |
| `HuntUnprepared` | no (returns before spending the arrow) | no | **candidate** |

`CraftMissingTool` and `ExperimentMissingTool` used to be one variant
(`CraftMissingTool`) logged from both `craft` and `experiment` with opposite
verdicts — `craft` catches it before spending (a candidate for removal),
`experiment` only after the combination is spent (a keeper). It was split
when the experiment message had to stop naming the recipe it would have
produced (`docs/recipe-tools.md`); the two justifications now live on two
variants.

## Not a gap: search's silent miss

`search()` on an all-miss roll logs nothing at all — `roll_found_items`
still runs (a real roll, possibly several independent draws, one per item
the tile offers), but if every one misses, the `for` loop over `found` has
nothing to iterate and no `EventKind` exists for "you search and find
nothing." This looks parallel to `hunt()`'s `HuntMissed` at a glance —
same shape, one logs, one doesn't — but it isn't: hunting spends an arrow
on every attempt, hit or miss, a real cost that deserves confirming even
when nothing is gained. Searching costs nothing beyond that one tile's own
temporary odds dip — the player can search the same tile again immediately,
for free, or move on and search another, indefinitely. Rule 2 doesn't apply
without a real cost, and rule 1 doesn't apply without a state change, so
this one is correctly silent. Worth writing down explicitly since it's
exactly the kind of case someone would otherwise flag as an inconsistency
later (as an earlier draft of this note did).

## Out of scope (for now)

- Acting on the five refusal-candidates above — removing their log lines.
  `TODO.md`'s "review events" line carries these as concrete follow-up items.
- A code-level mechanism that enforces this rule (e.g. `Game::log` refusing
  a call unless paired with an actual mutation) — worth considering if a
  future variant gets the classification wrong again, but not built now.
