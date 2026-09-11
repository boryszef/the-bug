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

  Scenario: Completing a quest unlocks the quest that depends on it
    Given a new game
    And the player has completed "The Digital Civilization"
    Then "The Digital Civilization" is completed
    And "Trouble in the East" is available
    And there is no active quest

  Scenario: Stock Up unlocks only once the axe quest is done
    Given a new game
    Then "Stock Up for Hard Times" is not available
    Given the player has completed "Trouble in the East"
    Then "Stock Up for Hard Times" is available

  Scenario: Crafting the target item completes the quest
    Given a new game
    And the player has completed "The Digital Civilization"
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
    And the player has completed "The Digital Civilization"
    And the player has accepted "Trouble in the East"
    And the player has 1 Branch in storage
    And the player has 1 Stone in storage
    And the player has 1 Cord in storage
    When the player experiments with 1 Branch, 1 Stone and 1 Cord
    Then "Trouble in the East" is completed

  Scenario: Crafting a different item does not advance the quest
    Given a new game
    And the player has completed "The Digital Civilization"
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

  Scenario: Accepting the axe quest unlocks Experiment and Items, but not yet Craft or Disassemble
    Given a new game
    And the player has completed "The Digital Civilization"
    Then "Experiment" is not unlocked
    And "Items" is not unlocked
    When the player accepts "Trouble in the East"
    Then "Experiment" is unlocked
    And "Items" is unlocked
    And "Craft" is not unlocked
    And "Disassemble" is not unlocked

  Scenario: Completing the axe quest unlocks Craft and Disassemble
    Given a new game
    And the player has completed "Trouble in the East"
    Then "Craft" is unlocked
    And "Disassemble" is unlocked
