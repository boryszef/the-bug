Feature: Disassembly

  Taking a carried item apart at the village returns the consumables of the
  recipe it decomposes into. Recovered parts always go to storage, whether
  the item came from storage or the equipment; craft-only recipes don't decompose.

  Scenario: Taking apart a stone axe returns its parts and logs
    Given a new game
    And the player has 1 Stone Axe in storage
    When the player disassembles "Stone Axe"
    Then storage contains 0 Stone Axe
    And storage contains 1 Branch
    And storage contains 1 Stone
    And storage contains 1 Cord
    And a disassembly of "Stone Axe" is logged

  Scenario: Disassembling an item from the equipment; the parts land in storage
    Given a new game
    And the player has 1 Stone Axe in the equipment
    When the player disassembles "Stone Axe"
    Then the equipment contains 0 Stone Axe
    And storage contains 1 Branch
    And storage contains 1 Stone
    And storage contains 1 Cord

  Scenario: Taking apart a scavenged umbrella yields fabric and a pole
    Given a new game
    And the player has 1 Umbrella in storage
    When the player disassembles "Umbrella"
    Then storage contains 0 Umbrella
    And storage contains 1 Fabric
    And storage contains 1 Pole

  Scenario: A craft-only item cannot be taken apart
    Given a new game
    And the player has 1 Arrow in storage
    When the player disassembles "Arrow"
    Then storage contains 1 Arrow
    And storage contains 0 Branch
    And no event was logged

  Scenario: Recovered parts stack onto existing storage
    Given a new game
    And the player has 1 Stone Axe in storage
    And the player has 2 Branch in storage
    When the player disassembles "Stone Axe"
    Then storage contains 3 Branch

  Scenario: Disassembling away from the village does nothing
    Given a new game
    And the player has 1 Stone Axe in storage
    And the player is away from the village
    When the player disassembles "Stone Axe"
    Then storage contains 1 Stone Axe
    And storage contains 0 Branch
    And no event was logged
