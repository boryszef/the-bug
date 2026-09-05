# Mini-backlog

## TODO

* craft menu should show what items are missing for a recipe
* map should not be random and certain terrain types should be clustered together (e.g. deadland, forest, mountains)
* crafting/experimenting/disassembly should only be possible at the village
* Players "bag" should have a limited size, when full, the player should take their findings to the village and deposit them in the storage

## DONE

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
