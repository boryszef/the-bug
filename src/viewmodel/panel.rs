//! Which panel occupies a front end's central area, and the fixed order the
//! `[` / `]` accelerators cycle it through. Shared by `gui::Panel` and
//! `tui::app::Panel` (formerly duplicated verbatim in both — see
//! docs/gui-panels.md) since the tab set and its order are UI-agnostic; each
//! front end still owns its own label lookup (`gui`'s `title_id`, `tui`'s
//! `footer-*` id mapping).

/// One of the five tabs a front end switches between. `ALL` is the single
/// source of truth for their order — `next`/`prev` cycle through it rather
/// than restating the order in a match.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Panel {
    #[default]
    Map,
    Experiment,
    Craft,
    Disassemble,
    Quests,
}

impl Panel {
    pub const ALL: [Panel; 5] = [
        Panel::Map,
        Panel::Experiment,
        Panel::Craft,
        Panel::Disassemble,
        Panel::Quests,
    ];

    /// The next tab in `ALL`, wrapping from the last back to the first.
    pub fn next(self) -> Panel {
        self.cycle(1)
    }

    /// The previous tab in `ALL`, wrapping from the first back to the last.
    pub fn prev(self) -> Panel {
        self.cycle(Self::ALL.len() - 1)
    }

    fn cycle(self, offset: usize) -> Panel {
        let index = Self::ALL
            .iter()
            .position(|&p| p == self)
            .expect("self is always one of ALL's variants");
        Self::ALL[(index + offset) % Self::ALL.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_cycles_through_every_panel_in_order_and_wraps() {
        assert_eq!(Panel::Map.next(), Panel::Experiment);
        assert_eq!(Panel::Experiment.next(), Panel::Craft);
        assert_eq!(Panel::Craft.next(), Panel::Disassemble);
        assert_eq!(Panel::Disassemble.next(), Panel::Quests);
        assert_eq!(Panel::Quests.next(), Panel::Map);
    }

    #[test]
    fn prev_cycles_through_every_panel_in_reverse_and_wraps() {
        assert_eq!(Panel::Map.prev(), Panel::Quests);
        assert_eq!(Panel::Quests.prev(), Panel::Disassemble);
        assert_eq!(Panel::Disassemble.prev(), Panel::Craft);
        assert_eq!(Panel::Craft.prev(), Panel::Experiment);
        assert_eq!(Panel::Experiment.prev(), Panel::Map);
    }

    #[test]
    fn next_and_prev_are_inverses_for_every_panel() {
        for panel in Panel::ALL {
            assert_eq!(panel.next().prev(), panel);
            assert_eq!(panel.prev().next(), panel);
        }
    }
}
