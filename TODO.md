# Mini-backlog

## TODO

* Experiment with 1 Branch and no Stone Axe produces message that reveals the recipe "you need a stone axe to make arrow"
* review events: event should reflect important messages coming from the game, not just reflect user actions - for example: finding an item should trigger an event, but trying to craft outside village should not. Also: a random roll merits a record even if it comes up empty, but only when the attempt had a real cost regardless of outcome (hunting always spends an arrow; searching costs nothing and is freely repeatable, so its silent miss is correct, not a gap) — docs/event-worthiness.md has the full rule and where every current EventKind stands against it
  * remove the log line for a pure refusal that changes nothing: UnknownRecipe, CraftShortage, CraftMissingTool (from craft() only - the experiment() trigger stays, since it happens after items are already spent), ExperimentShortage, HuntUnprepared
* EPIC: improve the map
  * implement roads and rivers
  * implement "bridge" POI - requires intersection of river and road, allows new items (eg. steel bolt, rusty metal)
* EPIC: more complex game mechanics
  * implement workshops as additional craftable locations alongside the Village (docs/village-crafting.md)
  * quests should have closing narrative, sometimes revealing part of the story, sometimes commenting the reward item
* EPIC: the story
  * add more quests
  * items should become enhancers and can be used in the game: axe allows to chop wood, metal detector improves the odds of finding metal-containing items (bow→hunt / arrows-spent done)

## DONE

* retire the ratatui `tui` front end — `src/tui/` deleted, `gui`/`tui` Cargo features and the two-config build gone; `gui` (egui) is the only front end (docs/adr/0004)
* functional-test suite — Gherkin scenarios run by cucumber-rs over game+viewmodel (shared core moved to `src/lib.rs`); six feature files (experiment/craft/disassemble/hunting/quests/bag-storage), ~40 behaviour scenarios migrated out of `src/game/mod.rs`'s unit tests (docs/functional-tests.md, docs/adr/0003)

* EPIC: more complex game mechanics
  * crafting/experimenting/disassembly should only be possible at the village — the gui disables the buttons away from it, no event is logged for the refusal (docs/village-crafting.md)
  * Players "bag" should have a limited size, when full, the player should take their findings to the village and deposit them in the storage or drop — Bag (limited, BAG_CAPACITY=50) split from Storage (unlimited, was the whole inventory); gui Items tab to manage both (docs/bag-and-storage.md)
* EPIC: the story
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
