use super::item::Item;
use super::map::Direction;
use super::quest::QuestID;
use super::recipe::{RECIPES, Recipe};
use std::collections::HashMap;
use std::io;

#[derive(Debug)]
pub struct Player {
    pub level: u32,
    pub experience: u32,
    /// Successful crafts so far — every tenth grants a point of experience.
    pub crafts_completed: u32,
    pub coordinates: (i32, i32),
    pub inventory: HashMap<Item, u32>,
    recipes: Vec<Recipe>,
    open_quest: Option<QuestID>,
    /// Occurrences of the open quest's condition seen since it was accepted.
    /// Meaningless while `open_quest` is `None`.
    quest_progress: u32,
    quests_completed: Vec<QuestID>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            level: 1,
            experience: 0,
            crafts_completed: 0,
            coordinates: (0, 0),
            inventory: HashMap::new(),
            recipes: Vec::new(),
            open_quest: None,
            quest_progress: 0,
            quests_completed: Vec::new(),
        }
    }
}

impl Player {
    /// The recipes the player has discovered, in discovery order.
    pub fn known_recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    /// `(recipes discovered, craftable recipes that exist)`. Disassemble-only
    /// recipes are decompositions, not recipes to discover, so they're left
    /// out of the total.
    pub fn recipe_progress(&self) -> (usize, usize) {
        let total = RECIPES.iter().filter(|r| r.craftable()).count();
        (self.known_recipes().len(), total)
    }

    /// The known recipe named `name`, if any.
    pub(super) fn find_known_recipe(&self, name: &str) -> Option<Recipe> {
        self.recipes.iter().find(|r| r.name() == name).copied()
    }

    /// Where `dir` takes the player, ignoring map boundaries — the caller
    /// checks those separately.
    pub(super) fn coordinates_after(&self, dir: Direction) -> (i32, i32) {
        let (dx, dy) = dir.delta();
        let (x, y) = self.coordinates;
        (x + dx, y + dy)
    }

    /// The currently active quest, if any.
    pub fn open_quest(&self) -> Option<QuestID> {
        self.open_quest
    }

    /// Occurrences of the open quest's condition seen so far. `0` when no
    /// quest is open.
    pub fn quest_progress(&self) -> u32 {
        self.quest_progress
    }

    /// Quests completed so far, in completion order.
    pub fn completed_quests(&self) -> &[QuestID] {
        &self.quests_completed
    }

    /// Sets quest state directly (used when loading a save).
    pub(crate) fn restore_quest_state(
        &mut self,
        open_quest: Option<QuestID>,
        quest_progress: u32,
        quests_completed: Vec<QuestID>,
    ) {
        self.open_quest = open_quest;
        self.quest_progress = quest_progress;
        self.quests_completed = quests_completed;
    }

    /// Opens `id` as the active quest, resetting progress to zero. Callers
    /// must have already validated `id` is acceptable.
    pub(super) fn open_quest_as(&mut self, id: QuestID) {
        self.open_quest = Some(id);
        self.quest_progress = 0;
    }

    /// Advances progress toward the open quest's condition by one,
    /// returning the new count. Meaningless if no quest is open.
    pub(super) fn advance_quest_progress(&mut self) -> u32 {
        self.quest_progress += 1;
        self.quest_progress
    }

    /// Closes the open quest, moving `id` into `completed_quests()` and
    /// resetting progress.
    pub(super) fn complete_quest(&mut self, id: QuestID) {
        self.open_quest = None;
        self.quest_progress = 0;
        self.quests_completed.push(id);
    }

    /// Removes `amount` of `item` from the inventory, dropping the entry
    /// entirely once it hits zero so exhausted items don't linger. Callers must
    /// have already checked the player holds enough — enforced in a debug
    /// build; a release build saturates at zero instead of wrapping, in case
    /// a caller's check turns out to have missed a case (e.g. a duplicated
    /// item in the amount being spent, checked as a whole but spent one
    /// entry at a time — see `spend_all`).
    pub(super) fn spend(&mut self, item: Item, amount: u32) {
        if let Some(remaining) = self.inventory.get_mut(&item) {
            debug_assert!(
                *remaining >= amount,
                "spend({item:?}, {amount}) exceeds the {remaining} held"
            );
            *remaining = remaining.saturating_sub(amount);
            if *remaining == 0 {
                self.inventory.remove(&item);
            }
        }
    }

    /// Removes each of `items` from the inventory. Callers must have already
    /// checked the player holds enough of every one.
    pub(super) fn spend_all(&mut self, items: &[(Item, u32)]) {
        for &(item, amount) in items {
            self.spend(item, amount);
        }
    }

    /// Adds `amount` of `item` to the inventory.
    pub(super) fn add_to_inventory(&mut self, item: Item, amount: u32) {
        *self.inventory.entry(item).or_insert(0) += amount;
    }

    /// Adds each of `items` to the inventory.
    pub(super) fn add_all_to_inventory(&mut self, items: &[(Item, u32)]) {
        for &(item, amount) in items {
            self.add_to_inventory(item, amount);
        }
    }

    /// How many of `item` the player currently holds.
    pub(super) fn inventory_count(&self, item: Item) -> u32 {
        self.inventory.get(&item).copied().unwrap_or(0)
    }

    /// Whether the player holds at least one of `item`.
    pub(super) fn has_item(&self, item: Item) -> bool {
        self.inventory_count(item) > 0
    }

    /// The first entry in `items` the player doesn't have enough of, as
    /// `(item, available, needed)`. `None` if every entry is satisfied.
    pub(super) fn first_shortage(&self, items: &[(Item, u32)]) -> Option<(Item, u32, u32)> {
        items.iter().find_map(|&(item, needed)| {
            let available = self.inventory_count(item);
            (available < needed).then_some((item, available, needed))
        })
    }

    /// The first tool in `tools` the player isn't holding. `None` if they hold
    /// every one (a single unit suffices — tools aren't consumed).
    pub(super) fn first_missing_tool(&self, tools: &[Item]) -> Option<Item> {
        tools.iter().copied().find(|&tool| !self.has_item(tool))
    }

    /// Adds `amount` to the player's experience.
    pub(super) fn add_experience(&mut self, amount: u32) {
        self.experience += amount;
    }

    /// Records a successful craft, granting a point of experience every
    /// tenth one.
    pub(super) fn record_successful_craft(&mut self) {
        self.crafts_completed += 1;
        if self.crafts_completed.is_multiple_of(10) {
            self.add_experience(1);
        }
    }

    /// Grants `xp` experience and each of `items` to the inventory.
    pub(super) fn grant_reward(&mut self, xp: u32, items: &[(Item, u32)]) {
        self.add_experience(xp);
        self.add_all_to_inventory(items);
    }

    /// Marks the craftable recipe with the given name as known (used when
    /// loading a save). Returns `false` for an unrecognised or non-craftable
    /// name, which the caller can ignore — a known recipe is always craftable.
    pub(crate) fn grant_recipe(&mut self, name: &str) -> bool {
        match RECIPES.iter().find(|recipe| recipe.name() == name).copied() {
            Some(recipe) if recipe.craftable() => {
                if !self.recipes.contains(&recipe) {
                    self.recipes.push(recipe);
                }
                true
            }
            _ => false,
        }
    }

    /// Marks `recipe` as known if it wasn't already, granting the discovery
    /// bonus. Returns whether it was newly learned (vs. already known).
    pub(super) fn learn_recipe(&mut self, recipe: Recipe) -> bool {
        let newly_learned = !self.recipes.contains(&recipe);
        if newly_learned {
            self.recipes.push(recipe);
            self.add_experience(10);
        }
        newly_learned
    }
}

impl super::SaveState for Player {
    type Saved = crate::save::PlayerState;

    fn save_state(&self) -> Self::Saved {
        crate::save::PlayerState {
            level: self.level,
            experience: self.experience,
            crafts_completed: self.crafts_completed,
            coordinates: self.coordinates,
            inventory: self.inventory.clone(),
            recipes: self
                .known_recipes()
                .iter()
                .map(|recipe| recipe.name().to_string())
                .collect(),
            open_quest: self.open_quest,
            quest_progress: self.quest_progress,
            quests_completed: self.quests_completed.clone(),
        }
    }
}

impl super::RestoreState for Player {
    type Saved = crate::save::PlayerState;

    fn restore_state(saved: Self::Saved) -> io::Result<Player> {
        let mut player = Player {
            level: saved.level,
            experience: saved.experience,
            crafts_completed: saved.crafts_completed,
            coordinates: saved.coordinates,
            inventory: saved.inventory,
            ..Player::default()
        };
        for name in &saved.recipes {
            player.grant_recipe(name);
        }
        player.restore_quest_state(
            saved.open_quest,
            saved.quest_progress,
            saved.quests_completed,
        );
        Ok(player)
    }
}
