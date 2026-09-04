### English strings. See docs/i18n-plan.md.
### This is the fallback locale: `i18n-embed-fl`'s `fl!()` macro checks every
### message id used in Rust against this file at compile time.

## Event log lines (src/game/event.rs's EventKind, rendered by i18n::event).

event-awoke = You wake up and decide to go for a walk.
event-found = You find a { $item } in the { $terrain }.
event-quest-accepted = Quest accepted: { $quest }
event-quest-completed = Quest complete: { $quest }!
event-unknown-recipe = You don't know how to craft a { $recipe }.
event-craft-shortage = You don't have enough { $needed } to craft a { $output }.
event-crafted = You craft a { $output }.
event-experiment-shortage = Experiment: { $items } → not enough { $missing } (have { $available }, need { $needed })
event-experiment-failed = Experiment: { $items } → nothing
event-experimented = Experiment: { $items } → { $output }{ $newly_learned ->
    [yes] { " (new recipe!)" }
   *[no] {""}
}
event-disassembled = You take apart a { $item }, recovering { $recovered }.

## Item names (nominative), src/game/item.rs's Item.

item-stick = Stick
item-stone = Stone
item-vine = Vine
item-cord = Cord
item-stone-axe = Stone Axe
item-arrow = Arrow
item-wooden-bow = Wooden Bow
item-plastic-bottle = Plastic Bottle
item-copper-wire = Copper Wire
item-coil = Coil
item-pole = Pole
item-microcontroller = Microcontroller
item-speaker = Speaker
item-metal-detector = Metal Detector
item-battery = Battery
item-solar-panel = Solar Panel
item-solar-charger = Solar Charger
item-circuit-board = Circuit Board
item-umbrella = Umbrella
item-fabric = Fabric

## Terrain names, src/game/map.rs's TerrainType.

terrain-meadow = Meadow
terrain-forest = Forest
terrain-cave = Cave
terrain-ruins = Ruins
terrain-village = Village
terrain-deadland = Deadland

## Quest name/description, src/game/quest.rs's QUESTS table.

quest-craft-arrows-name = Craft Arrows
quest-craft-arrows-description = Something has been stirring in the east again. Things had been quiet for a while, but trouble always seems to come from that direction. Whatever it is, it's spooked the big game, and a group of local hunters is gearing up for a hunt. They've asked you to craft 5 arrows for them. Head to the forest to gather sticks, then experiment with them to learn how arrows are made.
quest-explore-ruins-name = Explore the Ruins
quest-explore-ruins-description = A passing traveler mentioned some ruins nearby, said to be scattered with old artifacts. Apparently, a few villages once stood on these flats, remnants of a great civilization from roughly 500 years ago — the "Digital Civilization," as the explorers call it. You should go see it for yourself.

## Panel titles.

panel-map-title = Map
panel-player-title = Player
panel-events-title = Events
panel-craft-title = Craft
panel-recipes-title = Recipes
panel-experiment-title = Experiment
panel-available-title = Available
panel-selected-title = Selected
panel-disassemble-title = Disassemble
panel-items-title = Items
panel-quests-title = Quests
panel-active-title = Active

## Player panel.

player-level = Level: { $level }
player-xp = XP: { $xp }
player-recipes = Recipes: { $known }/{ $total }
player-quests = Quests: { $completed }/{ $total } — { $active }
player-quest-none = (none)
player-inventory = Inventory: { $items }
player-inventory-empty = (empty)

## Craft panel.

craft-empty = You haven't discovered any recipes yet.
craft-hint = ↑↓ move   Enter craft   Esc cancel

## Experiment panel.

experiment-hint = ↑↓ move   ←→ add/remove   Tab switch column   e run   Esc cancel

## Disassemble panel.

disassemble-empty = Nothing you're carrying can be taken apart.
disassemble-hint = ↑↓ move   Enter take apart   Esc cancel

## Quests panel.

quests-active-none = No quest accepted.
quests-active-blocked = Complete your active quest to accept another.
quests-available-empty = No quests available right now.
quests-completed-none = Completed: (none yet)
quests-completed = Completed: { $names }

## Footer key hints, one per panel (src/ui/app.rs's render_footer).

footer-map = ←↑↓→ move   s search   [ ] panel   q quit
footer-experiment = ↑↓ move   ←→ add/remove   Tab switch column   e run   [ ] panel   q quit
footer-craft = ↑↓ move   Enter craft   [ ] panel   q quit
footer-disassemble = ↑↓ move   Enter take apart   [ ] panel   q quit
footer-quests = ↑↓ move   Enter accept   [ ] panel   q quit
