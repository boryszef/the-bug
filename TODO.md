# Mini-backlog

## TODO

* review events: event should reflect important messages coming from the game, not just reflect user actions - for example: finding an item should trigger an event, but trying to craft outside village should not. Also: a random roll merits a record even if it comes up empty, but only when the attempt had a real cost regardless of outcome (hunting always spends an arrow; searching costs nothing and is freely repeatable, so its silent miss is correct, not a gap) — docs/event-worthiness.md has the full rule and where every current EventKind stands against it
  * remove the log line for a pure refusal that changes nothing: UnknownRecipe, CraftShortage, CraftMissingTool (from craft() only - the experiment() trigger stays, since it happens after items are already spent), ExperimentShortage, HuntUnprepared
* EPIC: the web build (docs/adr/0005-web-build.md)
  * a CI workflow — run cargo test / clippy / fmt on push (deploy.yml only builds+publishes)
* EPIC: improve the map
  * User sprite should be an icon too, not just a yellow dot.
  * implement roads and rivers
  * implement "bridge" POI - requires intersection of river and road, allows new items (eg. steel bolt, rusty metal)
* EPIC: more complex game mechanics
  * implement workshops as additional craftable locations alongside the Village (docs/village-crafting.md)
  * quests should have closing narrative, sometimes revealing part of the story, sometimes commenting the reward item
* EPIC: the story
  * add more quests

## DONE

* publish the web build — `.github/workflows/deploy.yml` builds with `trunk --release --public-url "/the-bug/"` and publishes `dist/` to GitHub Pages on a `v*` tag or manual dispatch; live at https://boryszef.github.io/the-bug/ (docs/adr/0005-web-build.md)
* web persistence + language — the browser build autosaves the game JSON to `localStorage["the-bug-game"]` (`App::save`, resumed in `app_creator`; a corrupt blob falls back to a fresh game) and reads the UI language from `navigator.language`; `save.rs` grew filesystem-free `to_json`/`from_json` (docs/adr/0005-web-build.md)
* WebAssembly build — the gui compiles to `wasm32` and runs in a browser (verified in headless Chrome); `#[cfg]`-split entry point + runner, `web_time::Instant`, getrandom wasm backend, `trunk` / `scripts/build-web.sh`; native path unchanged (docs/adr/0005-web-build.md)
* experiment "Looks good!" hint — a green line above the Run button when the current selection exactly matches an undiscovered recipe (`Player::experiment_would_discover`); names nothing else (docs/gui-panels.md, docs/recipe-tools.md)
* rename "Bag" → "Equipment" everywhere — `Player.equipment`, `equipment_*` methods, `EventKind::EquipmentFull`, the save key, the Items-tab label, `docs/equipment-and-storage.md`, cucumber steps. Pure rename, no behaviour change
* experiment tool-missing message no longer names the recipe — split `EventKind::ExperimentMissingTool { items }` off `CraftMissingTool`; experiment says only "you're missing a tool" (docs/recipe-tools.md)
* retire the ratatui `tui` front end — `src/tui/` deleted, `gui`/`tui` Cargo features and the two-config build gone; `gui` (egui) is the only front end (docs/adr/0004)
* functional-test suite — Gherkin scenarios run by cucumber-rs over game+viewmodel (shared core moved to `src/lib.rs`); six feature files (experiment/craft/disassemble/hunting/quests/equipment-storage), ~40 behaviour scenarios migrated out of `src/game/mod.rs`'s unit tests (docs/functional-tests.md, docs/adr/0003)

* EPIC: more complex game mechanics
  * crafting/experimenting/disassembly should only be possible at the village — the gui disables the buttons away from it, no event is logged for the refusal (docs/village-crafting.md)
  * Players carried pool should have a limited size, when full, the player should take their findings to the village and deposit them in the storage or drop — Equipment (limited, EQUIPMENT_BASE_CAPACITY=50; originally "Bag") split from Storage (unlimited, was the whole inventory); gui Items tab to manage both (docs/equipment-and-storage.md)
* EPIC: the story
  * items should become enhancers and can be used in the game: axe allows to chop wood, metal detector improves the odds of finding metal-containing items (bow→hunt / arrows-spent done; satchel→+50 equipment capacity done, docs/equipment-and-storage.md)
  * name of the quest should refer to the part of the story it tells, not the product or task
  * implement hunting: `h` on the Map tab, needs a bow held + spends an arrow, yields meat/bone/hide/fur on meadow/forest (docs/hunting.md)
* BUG: "Znajdujesz Plastikowa Butelka" -> "Znajdujesz Plastikową Butelkę" (accusative)
* BUG: arrow glyph in the event text does not render in egui (box instead of arrow)
* craft menu should show what items are missing for a recipe, eg. Coil: 10/2 Copper Wire, 0/1 Plastic Bottle (meaning "have/need item")
* EPIC: improve the map
  * add symbol representing the terrain (glyph) like in the TUI version on POI tiles (caves and ruins)
  * prevent sharp edges: tiles become aware of their neighbours, when a forrest has adjacent meadow tile, the meadow color trickles slightly into the forest tile; this will create an uneven, weavy border on the forest tile. The shape can be very simple and fixed for all tiles - it is just to prevent tiles from being so obviously square.
  * separate POI from terrain types - POI becomes an overlay, so the tile can be eg. forrest + cave and yields findings from both.
  * display text showing current terrain type and POI
  * Search button should have a shortcut `[s]` printed
  * higher quality POI symbols — procedural two-ink icons (arched cave, columned ruins, hut with a doorway)
  * raster POI icons — Cave/Ruins/Village drawn from `assets/icons/*.png` via `painter.image` (one cached egui texture each), replacing the procedural two-ink shapes (docs/icons.md, ADR 0001 amendment)
* generate the map with the mapgen clustering algorithm, moved in-crate to `src/mapgen/` and tuned by consts in `src/game/map.rs` (affinity 0.75); standalone `tools/mapgen` retired (docs/mapgen.md)
* front end chosen at build time: `gui` (default) / `tui` (legacy, frozen) Cargo features, mutually exclusive; `--gui` flag removed (docs/adr/0002)
* create alternative egui/eframe UI (map + movement + Experiment/Craft/Disassemble/Quests panels — parity with the tui; see docs/gui-frontend.md)
* implement the quest system: one quest unlocks other quests, completing a quest awards XP and items, quests also tell the story of the game world
* add i18n support (English + Polish, via Project Fluent)
* implement item disassembly (e.g. stone axe -> stick + stone + rope, umbrella -> pole + fabric)
* inventory panel: hide exhausted (zero) items and wrap the text
* show discovered-recipe ratio (known/total) in the player panel
* grant XP: +10 per new recipe, +1 per 10 crafts
* confirm search can yield multiple materials from one tile (test only)
* logs have categories (general / experiment / crafting), colour-coded
* stop logging walking; experiment logs now show inputs and result
* save game state (map, player, resources, recipes) to JSON; load with --load
* stamp the game's semantic version into the save file
* add time to event entries
* add Cave terrain type
* implement terrain probability
* implement search function
* implement resource probability
* add time decay to resource probability
* implement recipe system
* add crafting and experimenting to UI
* implement crafting system with stone axe (stick + stone + rope)
* add recipe discovery
