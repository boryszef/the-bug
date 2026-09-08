Feature: Crafting system

  Scenario: In a new game, user has no recipes
    Given a new game
    Then craft menu is empty

  Scenario: A recipe discovered by experiment appears in the craft menu
    Given a new game
    And the player has 2 Vine in storage
    When the player experiments with 2 Vine
    Then storage contains 1 Cord
    And storage contains 0 Vine
    And a "Cord" craft is offered in the craft menu

  Scenario: Experiment fails with insufficient materials, but material is used
    Given a new game
    And the player has 1 Vine in storage
    When the player experiments with 1 Vine
    Then storage contains 0 Vine
    And the storage contains 0 Cord
    And craft menu is empty

  Scenario: Experiment fails when too much material used, but material is used
    Given a new game
    And the player has 3 Vine in storage
    When the player experiments with 3 Vine
    Then storage contains 0 Vine
    And the storage contains 0 Cord
    And craft menu is empty

  Scenario: Experiment with a tool required
    Given a new game
    And the player has 1 Stone Axe in storage
    And the player has 1 Branch in storage
    When the player experiments with 1 Branch
    Then the storage contains 1 Arrow
    And the storage contains 0 Branch
    And the storage contains 1 Stone Axe
    And an "Arrow" craft is offered in the craft menu

  Scenario: Experiment with a tool missing
    Given a new game
    And the player has 1 Branch in storage
    When the player experiments with 1 Branch
    Then the storage contains 0 Arrow
    And the storage contains 0 Branch
    And the storage contains 0 Stone Axe
    And craft menu is empty
