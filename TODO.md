# Mini-backlog

## TODO

* EPIC: improve the map
  * implement roads and rivers
  * implement "bridge" POI - requires intersection of river and road, allows new items (eg. steel bolt, rusty metal)
* EPIC: reach game mechanics
  * crafting/experimenting/disassembly should only be possible at the village
  * Players "bag" should have a limited size, when full, the player should take their findings to the village and deposit them in the storage
* EPIC: the story
  * add more quests
  * items should become enhancers and can be used in the game: axe allows to chop wood, metal detector improves the odds of finding metal-containing items (bow→hunt / arrows-spent done)

## DONE

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
