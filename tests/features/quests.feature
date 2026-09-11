Feature: Quests

  One quest is active at a time. A quest becomes available once every quest
  it depends on is in the completed list; accepting it takes it off the
  available list; crafting or experimenting its target item completes it,
  clearing the active slot.

  Scenario: A new game offers only the opening quest, and it can be accepted
    Given a new game
    Then "The Digital Civilization" is available
    And "Trouble in the East" is not available
    And there is no active quest
    When the player accepts "The Digital Civilization"
    Then "The Digital Civilization" is accepted
    And the quest progress is 0

  Scenario: Only one quest is active at a time
    Given a new game
    And the player has accepted "The Digital Civilization"
    When the player accepts "Trouble in the East"
    Then accepting "Trouble in the East" is refused because another quest is active

  Scenario: A quest with unmet prerequisites cannot be accepted
    Given a new game
    When the player accepts "Trouble in the East"
    Then accepting "Trouble in the East" is refused because its prerequisites aren't met

  Scenario: Accepting the opening quest removes it from the available list
    Given a new game
    And the player has accepted "The Digital Civilization"
    Then no quest is available

  Scenario: Completing the ruins quest unlocks the search for old artifacts
    Given a new game
    And the player has completed "The Digital Civilization"
    Then "The Digital Civilization" is completed
    And "What the Ruins Kept" is available
    And there is no active quest

  Scenario: Completing the search quest unlocks the cord quest
    Given a new game
    And the player has completed "What the Ruins Kept"
    Then "What the Ruins Kept" is completed
    And "Follow the Thread" is available
    And there is no active quest

  Scenario: Completing the cord quest unlocks the axe quest
    Given a new game
    And the player has completed "Follow the Thread"
    Then "Follow the Thread" is completed
    And "Trouble in the East" is available
    And there is no active quest

  Scenario: Completing the axe quest unlocks the umbrella quest
    Given a new game
    And the player has completed "Trouble in the East"
    Then "Trouble in the East" is completed
    And "One Man's Trash" is available
    And there is no active quest

  Scenario: Stock Up unlocks only once the umbrella quest is done
    Given a new game
    Then "Stock Up for Hard Times" is not available
    Given the player has completed "One Man's Trash"
    Then "Stock Up for Hard Times" is available

  Scenario: Crafting the target item completes the quest
    Given a new game
    And the player has completed "Follow the Thread"
    And the player has accepted "Trouble in the East"
    And the player knows the "Stone Axe" recipe
    And the player has 1 Branch in storage
    And the player has 1 Stone in storage
    And the player has 1 Cord in storage
    When the player crafts "Stone Axe"
    Then "Trouble in the East" is completed
    And there is no active quest

  Scenario: Experimenting the target item also completes the quest
    Given a new game
    And the player has completed "Follow the Thread"
    And the player has accepted "Trouble in the East"
    And the player has 1 Branch in storage
    And the player has 1 Stone in storage
    And the player has 1 Cord in storage
    When the player experiments with 1 Branch, 1 Stone and 1 Cord
    Then "Trouble in the East" is completed

  Scenario: Crafting a different item does not advance the quest
    Given a new game
    And the player has completed "Follow the Thread"
    And the player has accepted "Trouble in the East"
    And the player knows the "Cord" recipe
    And the player has 2 Vine in storage
    When the player crafts "Cord"
    Then the active quest is "Trouble in the East"
    And the quest progress is 0

  Scenario: Accepting the opening quest unlocks the map
    Given a new game
    Then "Map" is not unlocked
    When the player accepts "The Digital Civilization"
    Then "Map" is unlocked

  Scenario: Accepting the search quest unlocks Items
    Given a new game
    And the player has completed "The Digital Civilization"
    Then "Items" is not unlocked
    When the player accepts "What the Ruins Kept"
    Then "Items" is unlocked
    And "Experiment" is not unlocked
    And "Craft" is not unlocked
    And "Disassemble" is not unlocked

  Scenario: Accepting the cord quest unlocks Experiment
    Given a new game
    And the player has completed "What the Ruins Kept"
    Then "Experiment" is not unlocked
    When the player accepts "Follow the Thread"
    Then "Experiment" is unlocked
    And "Craft" is not unlocked
    And "Disassemble" is not unlocked

  Scenario: Completing the cord quest unlocks Craft
    Given a new game
    And the player has completed "What the Ruins Kept"
    And the player has accepted "Follow the Thread"
    And the player has 2 Vine in storage
    Then "Craft" is not unlocked
    When the player experiments with 2 Vine
    Then "Follow the Thread" is completed
    And "Craft" is unlocked

  Scenario: Accepting the umbrella quest unlocks Disassemble
    Given a new game
    And the player has completed "Trouble in the East"
    Then "Disassemble" is not unlocked
    When the player accepts "One Man's Trash"
    Then "Disassemble" is unlocked

  Scenario: Disassembling the target item completes the umbrella quest
    Given a new game
    And the player has completed "Trouble in the East"
    And the player has accepted "One Man's Trash"
    And the player has 1 Umbrella in storage
    When the player disassembles "Umbrella"
    Then "One Man's Trash" is completed
    And there is no active quest
