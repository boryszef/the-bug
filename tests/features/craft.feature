Feature: Crafting

  Building a known recipe at the village: consumables come from storage first
  and the equipment for any remainder, a required tool is checked but never spent,
  and an unknown recipe or a missing tool is refused.

  Scenario: Crafting consumes exactly its consumables and logs
    Given a new game
    And the player knows the "Cord" recipe
    And the player has 2 Vine in storage
    When the player crafts "Cord"
    Then storage contains 0 Vine
    And storage contains 1 Cord
    And a craft of "Cord" is logged

  Scenario: A craft draws from storage before the equipment
    Given a new game
    And the player knows the "Cord" recipe
    And the player has 1 Vine in storage
    And the player has 1 Vine in the equipment
    When the player crafts "Cord"
    Then storage contains 0 Vine
    And the equipment contains 0 Vine
    And storage contains 1 Cord

  Scenario: A craft leaves the equipment alone when storage alone covers it
    Given a new game
    And the player knows the "Cord" recipe
    And the player has 2 Vine in storage
    And the player has 5 Vine in the equipment
    When the player crafts "Cord"
    Then storage contains 0 Vine
    And the equipment contains 5 Vine

  Scenario: Crafting is blocked without the required tool held
    Given a new game
    And the player knows the "Wooden Bow" recipe
    And the player has 1 Branch in storage
    And the player has 1 Cord in storage
    When the player crafts "Wooden Bow"
    Then storage contains 0 Wooden Bow
    And storage contains 1 Branch
    And storage contains 1 Cord
    And the craft is refused for want of a "Stone Axe"

  Scenario: A craft keeps the tool it used
    Given a new game
    And the player knows the "Wooden Bow" recipe
    And the player has 1 Branch in storage
    And the player has 1 Cord in storage
    And the player has 1 Stone Axe in storage
    When the player crafts "Wooden Bow"
    Then storage contains 1 Wooden Bow
    And storage contains 0 Branch
    And storage contains 0 Cord
    And storage contains 1 Stone Axe

  Scenario: Crafting away from the village does nothing
    Given a new game
    And the player knows the "Cord" recipe
    And the player has 2 Vine in storage
    And the player is away from the village
    When the player crafts "Cord"
    Then storage contains 2 Vine
    And storage contains 0 Cord
    And no event was logged

  Scenario: Crafting an unknown recipe is refused
    Given a new game
    When the player crafts "Cord"
    Then the craft is refused as an unknown recipe
