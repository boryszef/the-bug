### English strings. See docs/i18n-plan.md.
### This is the fallback locale: `i18n-embed-fl`'s `fl!()` macro checks every
### message id used in Rust against this file at compile time.

## Event log lines (src/game/event.rs's EventKind, rendered by i18n::event).

event-awoke = You wake up and decide to go for a walk.
event-found = You find a { $item } in the { $terrain }.
event-found-poi = You find a { $item } in the { $poi }.
event-quest-accepted = Quest accepted: { $quest }
event-quest-completed = Quest complete: { $quest }!
event-crafted = You craft a { $output }.
event-experiment-failed = Experiment: { $items } -> nothing
event-experiment-missing-tool = Experiment: { $items } -> you're missing a tool
event-experimented = Experiment: { $items } -> { $output }{ $newly_learned ->
    [yes] { " (new recipe!)" }
   *[no] {""}
}
event-disassembled = You take apart a { $item }, recovering { $recovered }.
event-hunted = Hunt: { $items }.
event-hunt-missed = Hunt: the quarry got away.
event-equipment-full = You have no room left for the { $item }.
event-dropped = You drop a { $item }.
event-leveled-up = You levelled up! You are now level { $level }.

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

quest-explore-ruins-name = The Digital Civilization
quest-explore-ruins-description = A passing traveler mentioned some ruins nearby, said to be scattered with old artifacts. Apparently, a few villages once stood on these flats, remnants of a great civilization from roughly 500 years ago — the "Digital Civilization," as the explorers call it. You should go see it for yourself.
quest-old-civilization-name = What the Ruins Kept
quest-old-civilization-description = The ruins go deeper than they first looked — foundations, walls, whole rooms half-swallowed by vegetation. The Digital Civilization ran on electricity, villagers say, and on devices that thought a little for you — boxes and slabs and screens in every shape, wired into everything they built. Most of it is dead now, corroded past telling one machine from another. Dig around and see what the ground is still holding onto. Copper wire would be a start; every tinkerer in the village wants some.
quest-craft-cord-name = Follow the Thread
quest-craft-cord-description = Nobody in the settlement bothers with electricity anymore; what people know now is thread and knot, field and forge. There's an old wariness about the devices in the ruins, too — something about not trusting the things that used to think for you, though nobody can say why, not really, not anymore. Still, you can't help noticing the thin, shiny panels scattered among the wreckage, smooth as still water under all that dirt. Those shiny wires look atractive, but this is not what you need — a length of good cord. Vine's easy enough to gather; try combining a few lengths and see what holds together. Sometimes one thread really does lead to another.
quest-craft-axe-name = Trouble in the East
quest-craft-axe-description = Something has been stirring in the east again. Things had been quiet for a while, but trouble always seems to come from that direction. The old folk have stories about why — how the first settlers had to fight the land itself to survive, how even the plants seemed to want them gone. Some claim the old forest could think, could plan, could wait. Nonsense, most people say, shaking their heads — plants don't scheme. But you find yourself wondering, all the same. Whatever's stirring out there, you'd better be prepared. Head to the forest, the meadow and the cave, gather materials, then experiment with them to learn how to craft a stone axe.
quest-disassemble-umbrella-name = One Man's Trash
quest-disassemble-umbrella-description = An old umbrella, broken beyond fixing, still has good parts in it — fabric, a metal pole, maybe more once you look closely. You've been picking apart everything the ruins give up lately, just to see how it was put together and what's worth keeping. Old technology has a way of hiding something useful inside, if you're willing to take it apart properly rather than leave it whole and useless. There's bound to be an umbrella buried somewhere out there.
quest-stock-up-name = Stock Up for Hard Times
quest-stock-up-description = An old hand at the settlement keeps eyeing the sky. The cold months are closing in, she says, and the stores won't see everyone through — meat, hide, bone, fur, whatever you can bring back before the frost sets in. She also tells you to keep your eyes open out there: the meadow and forest aren't quite what the old stories describe, not anymore, though she couldn't tell you exactly what's changed. Take your bow. Five good hunts should be a start.

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

## Items panel (the Equipment/Inventory split; see
## docs/equipment-and-storage.md). "panel-items-title" above doubles as this
## tab's heading.

items-equipment-title = Equipment: { $count }/{ $capacity }
items-storage-title = Inventory
items-equipment-empty = Your equipment is empty.
items-storage-empty = Storage is empty.

## Experiment panel.

# Shown while building a selection that exactly matches a recipe the player
# hasn't discovered yet. Deliberately says nothing more — see docs/recipe-tools.md.
experiment-promising = Looks good!

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
action-settings = Settings
action-theme-light = Light
action-theme-dark = Dark
font-size-label = Font:
language-label = Language:
font-size-small = Small
font-size-medium = Medium
font-size-large = Large
font-size-extra-large = Extra Large
gui-hint = [ ] switch tabs   q quit
gui-hint-map = drag to pan   scroll to zoom
