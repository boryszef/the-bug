Feature: Hunting

  A hunt needs a Wooden Bow and an Arrow carried in the equipment. Missing either
  is refused before anything is rolled or spent, and gear sitting in storage
  back at the village doesn't count.

  Scenario: Hunting with no bow is refused and spends nothing
    Given a new game
    And the player has 3 Arrow in the equipment
    When the player hunts
    Then no event was logged
    And the equipment contains 3 Arrow

  Scenario: Hunting with a bow but no arrows is refused
    Given a new game
    And the player has 1 Wooden Bow in the equipment
    When the player hunts
    Then no event was logged

  Scenario: Hunting gear must be carried, not left in storage
    Given a new game
    And the player has 1 Wooden Bow in storage
    And the player has 3 Arrow in storage
    When the player hunts
    Then no event was logged
