//! Which panel occupies the front end's central area, and the fixed order the
//! `[` / `]` accelerators cycle it through. The tab set and its order are
//! UI-agnostic, so they live here; `gui` owns only its own label lookup
//! (`title_id`). See docs/gui-panels.md and docs/tutorial-unlocks.md.

use crate::game::Unlocked;

/// One of the five tabs a front end switches between. `ALL` is the single
/// source of truth for their order — `next`/`prev` cycle through it rather
/// than restating the order in a match.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Panel {
    #[default]
    Quests,
    Map,
    Items,
    Experiment,
    Craft,
    Disassemble,
}

impl Panel {
    pub const ALL: [Panel; 6] = [
        Panel::Quests,
        Panel::Map,
        Panel::Items,
        Panel::Experiment,
        Panel::Craft,
        Panel::Disassemble,
    ];

    /// The [`Unlocked`] feature that must be present for this tab to be
    /// shown — `None` for a tab that's always visible. See
    /// docs/tutorial-unlocks.md for the schedule that grants each one.
    fn required_unlock(self) -> Option<Unlocked> {
        match self {
            Panel::Quests => None,
            Panel::Map => Some(Unlocked::Map),
            Panel::Items => Some(Unlocked::Items),
            Panel::Experiment => Some(Unlocked::Experiment),
            Panel::Craft => Some(Unlocked::Craft),
            Panel::Disassemble => Some(Unlocked::Disassemble),
        }
    }

    /// Whether this tab should be shown, given the player's `unlocked`
    /// features.
    pub fn is_visible(self, unlocked: &[Unlocked]) -> bool {
        self.required_unlock()
            .is_none_or(|required| unlocked.contains(&required))
    }

    /// `ALL`, filtered down to the tabs `unlocked` makes visible — the tab
    /// bar's contents, in order.
    pub fn visible(unlocked: &[Unlocked]) -> Vec<Panel> {
        Self::ALL
            .into_iter()
            .filter(|panel| panel.is_visible(unlocked))
            .collect()
    }

    /// The next visible tab in `ALL`, wrapping around. Falls back to `self`
    /// if nothing else is visible.
    pub fn next(self, unlocked: &[Unlocked]) -> Panel {
        self.cycle(1, unlocked)
    }

    /// The previous visible tab in `ALL`, wrapping around. Falls back to
    /// `self` if nothing else is visible.
    pub fn prev(self, unlocked: &[Unlocked]) -> Panel {
        self.cycle(Self::ALL.len() - 1, unlocked)
    }

    /// Walks `ALL` from `self` by `offset` steps at a time, wrapping,
    /// stopping at the first visible tab it lands on — or back at `self` if
    /// none of the others are visible. Indexes into `ALL` (always valid,
    /// since `self` is one of its variants) rather than into the filtered
    /// `visible` set, so this never depends on `self` itself already being
    /// visible.
    fn cycle(self, offset: usize, unlocked: &[Unlocked]) -> Panel {
        let start = Self::ALL
            .iter()
            .position(|&p| p == self)
            .expect("self is always one of ALL's variants");
        let mut index = start;
        loop {
            index = (index + offset) % Self::ALL.len();
            let candidate = Self::ALL[index];
            if candidate.is_visible(unlocked) || index == start {
                return candidate;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every tab unlocked: cycling behaves exactly like the plain `ALL`
    /// order.
    const EVERYTHING: [Unlocked; 5] = [
        Unlocked::Map,
        Unlocked::Experiment,
        Unlocked::Craft,
        Unlocked::Disassemble,
        Unlocked::Items,
    ];

    #[test]
    fn next_cycles_through_every_panel_in_order_and_wraps() {
        assert_eq!(Panel::Quests.next(&EVERYTHING), Panel::Map);
        assert_eq!(Panel::Map.next(&EVERYTHING), Panel::Items);
        assert_eq!(Panel::Items.next(&EVERYTHING), Panel::Experiment);
        assert_eq!(Panel::Experiment.next(&EVERYTHING), Panel::Craft);
        assert_eq!(Panel::Craft.next(&EVERYTHING), Panel::Disassemble);
        assert_eq!(Panel::Disassemble.next(&EVERYTHING), Panel::Quests);
    }

    #[test]
    fn prev_cycles_through_every_panel_in_reverse_and_wraps() {
        assert_eq!(Panel::Quests.prev(&EVERYTHING), Panel::Disassemble);
        assert_eq!(Panel::Disassemble.prev(&EVERYTHING), Panel::Craft);
        assert_eq!(Panel::Craft.prev(&EVERYTHING), Panel::Experiment);
        assert_eq!(Panel::Experiment.prev(&EVERYTHING), Panel::Items);
        assert_eq!(Panel::Items.prev(&EVERYTHING), Panel::Map);
        assert_eq!(Panel::Map.prev(&EVERYTHING), Panel::Quests);
    }

    #[test]
    fn next_and_prev_are_inverses_for_every_panel() {
        for panel in Panel::ALL {
            assert_eq!(panel.next(&EVERYTHING).prev(&EVERYTHING), panel);
            assert_eq!(panel.prev(&EVERYTHING).next(&EVERYTHING), panel);
        }
    }

    #[test]
    fn with_nothing_unlocked_only_quests_is_visible_and_cycling_stays_put() {
        assert_eq!(Panel::visible(&[]), [Panel::Quests]);
        assert_eq!(Panel::Quests.next(&[]), Panel::Quests);
        assert_eq!(Panel::Quests.prev(&[]), Panel::Quests);
    }

    #[test]
    fn cycling_skips_locked_panels() {
        let unlocked = [Unlocked::Map];
        assert_eq!(Panel::visible(&unlocked), [Panel::Quests, Panel::Map]);
        assert_eq!(Panel::Map.next(&unlocked), Panel::Quests);
        assert_eq!(Panel::Quests.next(&unlocked), Panel::Map);
        assert_eq!(Panel::Map.prev(&unlocked), Panel::Quests);
        assert_eq!(Panel::Quests.prev(&unlocked), Panel::Map);
    }

    #[test]
    fn default_panel_is_quests_since_nothing_else_is_unlocked_at_the_start() {
        assert_eq!(Panel::default(), Panel::Quests);
    }
}
