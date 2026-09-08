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

  Scenario: An experiment draws from storage before the bag
    Given a new game
    And the player has 1 Vine in storage
    And the player has 1 Vine in the bag
    When the player experiments with 2 Vine
    Then storage contains 0 Vine
    And the bag contains 0 Vine
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

  Scenario: Experiment matching a recipe without its tool learns nothing
    Given a new game
    And the player has 1 Branch in storage
    When the player experiments with 1 Branch
    Then storage contains 0 Arrow
    And storage contains 0 Branch
    And storage contains 0 Stone Axe
    And the recipe "Arrow" is not known
    And craft menu is empty

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
