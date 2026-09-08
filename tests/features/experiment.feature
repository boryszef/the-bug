Feature: Experimenting

  Combining items at the village to discover a recipe: a match learns the
  recipe and builds its output, a non-match still spends the inputs, and a
  match whose tool isn't held learns nothing.

  Scenario: In a new game the player knows no recipes
    Given a new game
    Then craft menu is empty

  Scenario: A recipe discovered by experiment appears in the craft menu
    Given a new game
    And the player has 2 Vine in storage
    When the player experiments with 2 Vine
    Then storage contains 1 Cord
    And storage contains 0 Vine
    And a "Cord" craft is offered in the craft menu

  Scenario: Experiment fails with insufficient materials, but the materials are still used
    Given a new game
    And the player has 1 Vine in storage
    When the player experiments with 1 Vine
    Then storage contains 0 Vine
    And storage contains 0 Cord
    And craft menu is empty

  Scenario: Experiment fails when too much material is used, but the material is still used
    Given a new game
    And the player has 3 Vine in storage
    When the player experiments with 3 Vine
    Then storage contains 0 Vine
    And storage contains 0 Cord
    And craft menu is empty

  Scenario: An experiment draws from storage before the equipment
    Given a new game
    And the player has 1 Vine in storage
    And the player has 1 Vine in the equipment
    When the player experiments with 2 Vine
    Then storage contains 0 Vine
    And the equipment contains 0 Vine
    And a "Cord" craft is offered in the craft menu

  Scenario: Discovering a recipe grants experience; rediscovering it does not
    Given a new game
    And the player has 4 Vine in storage
    When the player experiments with 2 Vine
    Then the player has 10 experience
    When the player experiments with 2 Vine
    Then the player has 10 experience

  Scenario: Experiment with the required tool held discovers and builds
    Given a new game
    And the player has 1 Stone Axe in storage
    And the player has 1 Branch in storage
    When the player experiments with 1 Branch
    Then storage contains 1 Arrow
    And storage contains 0 Branch
    And storage contains 1 Stone Axe
    And an "Arrow" craft is offered in the craft menu

  Scenario: Experiment matching a recipe without its tool learns nothing and gives nothing away
    Given a new game
    And the player has 1 Branch in storage
    When the player experiments with 1 Branch
    Then storage contains 0 Arrow
    And storage contains 0 Branch
    And storage contains 0 Stone Axe
    And the recipe "Arrow" is not known
    And craft menu is empty
    And the experiment reports only that a tool is missing

  Scenario: The exact components of an undiscovered recipe are flagged as promising
    Given a new game
    Then combining 2 Vine is promising
    And combining 1 Vine is not promising
    And combining 3 Vine is not promising
    And combining 1 Battery and 1 Speaker is not promising

  Scenario: A combination for an already-known recipe is not flagged
    Given a new game
    And the player knows the "Cord" recipe
    Then combining 2 Vine is not promising

  Scenario: A disassemble-only item cannot be experimented into existence
    Given a new game
    And the player has 1 Battery in storage
    And the player has 1 Speaker in storage
    When the player experiments with 1 Battery and 1 Speaker
    Then storage contains 0 Battery
    And storage contains 0 Speaker
    And storage contains 0 Electronic Toy
    And the recipe "Electronic Toy" is not known

  Scenario: Experimenting away from the village does nothing
    Given a new game
    And the player has 2 Vine in storage
    And the player is away from the village
    When the player experiments with 2 Vine
    Then storage contains 2 Vine
    And the recipe "Cord" is not known
    And craft menu is empty
    And no event was logged
