Feature: Equipment and storage transfers

  The village Storage is unlimited; the Equipment the player carries is
  limited. Moving items between the two, and dropping from storage, only work
  at the village; dropping from the equipment works anywhere.

  Scenario: Moving items from the equipment to storage needs the village
    Given a new game
    And the player has 3 Vine in the equipment
    And the player is away from the village
    When the player transfers 3 Vine to storage
    Then the equipment contains 3 Vine
    And storage contains 0 Vine

  Scenario: At the village, items move from the equipment to storage
    Given a new game
    And the player has 3 Vine in the equipment
    When the player transfers 3 Vine to storage
    Then the equipment contains 0 Vine
    And storage contains 3 Vine

  Scenario: Moving items from storage to the equipment needs the village
    Given a new game
    And the player has 3 Vine in storage
    And the player is away from the village
    When the player transfers 3 Vine to the equipment
    Then storage contains 3 Vine
    And the equipment contains 0 Vine

  Scenario: At the village, items move from storage to the equipment
    Given a new game
    And the player has 3 Vine in storage
    When the player transfers 3 Vine to the equipment
    Then storage contains 0 Vine
    And the equipment contains 3 Vine

  Scenario: Dropping from the equipment works anywhere
    Given a new game
    And the player has 3 Vine in the equipment
    And the player is away from the village
    When the player drops 1 Vine from the equipment
    Then the equipment contains 2 Vine
    And a drop of "Vine" is logged

  Scenario: Dropping from storage needs the village
    Given a new game
    And the player has 3 Vine in storage
    And the player is away from the village
    When the player drops 1 Vine from storage
    Then storage contains 3 Vine
    And no event was logged

  Scenario: At the village, dropping from storage removes one unit and logs
    Given a new game
    And the player has 3 Vine in storage
    When the player drops 1 Vine from storage
    Then storage contains 2 Vine
    And a drop of "Vine" is logged

  Scenario: A near-full equipment rejects a transfer that would overflow it
    Given a new game
    And the player has 50 Vine in storage
    And the player has 1 Branch in the equipment
    When the player transfers 50 Vine to the equipment
    Then storage contains 50 Vine
    And the equipment contains 0 Vine

  Scenario: A Satchel in the equipment makes room for that same transfer
    Given a new game
    And the player has 50 Vine in storage
    And the player has 1 Satchel in the equipment
    When the player transfers 50 Vine to the equipment
    Then storage contains 0 Vine
    And the equipment contains 50 Vine
