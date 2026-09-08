### English strings. See docs/i18n-plan.md.
### This is the fallback locale: `i18n-embed-fl`'s `fl!()` macro checks every
### message id used in Rust against this file at compile time.

## Event log lines (src/game/event.rs's EventKind, rendered by i18n::event).

event-awoke = You wake up and decide to go for a walk.
event-found = You find a { $item } in the { $terrain }.
event-found-poi = You find a { $item } in the { $poi }.
event-quest-accepted = Quest accepted: { $quest }
event-quest-completed = Quest complete: { $quest }!
event-unknown-recipe = You don't know how to craft a { $recipe }.
event-craft-shortage = You don't have enough { $needed } to craft a { $output }.
event-craft-missing-tool = You need a { $tool } to craft a { $output }.
event-crafted = You craft a { $output }.
event-experiment-shortage = Experiment: { $items } -> not enough { $missing } (have { $available }, need { $needed })
event-experiment-failed = Experiment: { $items } -> nothing
event-experimented = Experiment: { $items } -> { $output }{ $newly_learned ->
    [yes] { " (new recipe!)" }
   *[no] {""}
}
event-disassembled = You take apart a { $item }, recovering { $recovered }.
event-hunted = Hunt: { $items }.
event-hunt-missed = Hunt: the quarry got away.
event-hunt-unprepared = You need a { $missing } to hunt.
event-bag-full = Your bag is too full to carry the { $item }.
event-dropped = You drop a { $item }.

## Item names (nominative), src/game/item.rs's Item.

item-branch = Branch
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
item-electronic-toy = Electronic Toy
item-rusty-metal = Rusty Metal
item-metal-knife = Metal Knife
item-electric-motor = Electric Motor
item-steel-bolt = Steel Bolt
item-meat = Meat
item-bone = Bone
item-hide = Hide
item-fur = Fur
item-satchel = Satchel
item-bone-needle = Bone Needle

## Terrain names, src/game/map.rs's TerrainType.

terrain-meadow = Meadow
terrain-forest = Forest
terrain-deadland = Deadland

## Point-of-interest names, src/game/map.rs's Poi.

poi-cave = Cave
poi-ruins = Ruins
poi-village = Village

## Quest name/description, src/game/quest.rs's QUESTS table.

quest-craft-axe-name = Trouble in the East
quest-craft-axe-description = Something has been stirring in the east again. Things had been quiet for a while, but trouble always seems to come from that direction. Whatever it is, you'd better be prepared! Head to the forest, the meadow and the cave, gather materials, then experiment with them to learn how craft a stone axe.
quest-explore-ruins-name = The Digital Civilization
quest-explore-ruins-description = A passing traveler mentioned some ruins nearby, said to be scattered with old artifacts. Apparently, a few villages once stood on these flats, remnants of a great civilization from roughly 500 years ago — the "Digital Civilization," as the explorers call it. You should go see it for yourself.
quest-stock-up-name = Stock Up for Hard Times
quest-stock-up-description = An old hand at the settlement keeps eyeing the sky. The cold months are closing in, she says, and the stores won't see everyone through. Take your bow to the meadows and the forest and bring back what you can — meat, hide, bone, fur. Five good hunts should be a start.

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
panel-completed-title = Completed

## Player panel.

player-level = Level: { $level }
player-xp = XP: { $xp }
player-recipes = Recipes: { $known }/{ $total }
player-equipment = Equipment: { $carried }/{ $capacity }
player-quests = Quests: { $completed }/{ $total } — { $active }
player-quest-none = (none)

## Village-gated actions: the gui disables the Craft/Experiment/Disassemble
## buttons away from the Village and shows this hint instead — no event is
## logged (a refusal here is the player's own doing, not something the game
## did, so it isn't event-log material; see docs/village-crafting.md).

village-required-hint = You need to be in the Village to do this.

## Items panel (the Bag/Inventory split; see
## docs/bag-and-storage.md). "panel-items-title" above doubles as this
## tab's heading.

items-bag-title = Bag: { $count }/{ $capacity }
items-storage-title = Inventory
items-bag-empty = Your bag is empty.
items-storage-empty = Storage is empty.

## Craft panel.

craft-empty = You haven't discovered any recipes yet.

## Disassemble panel.

disassemble-empty = Nothing you're carrying can be taken apart.

## Quests panel.

quests-active-none = No quest accepted.
quests-active-blocked = Complete your active quest to accept another.
quests-available-empty = No quests available right now.
quests-completed-empty = No quests completed yet.

## Controls (src/gui/mod.rs's toolbar, panel buttons, hint bar).

action-quit = Quit (q)
action-experiment = Run experiment
action-accept = Accept
action-search = Search (s)
action-hunt = Hunt (h)
gui-hint = [ ] switch tabs   q quit
gui-hint-map = drag to pan   scroll to zoom
