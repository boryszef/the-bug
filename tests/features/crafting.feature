Feature: Crafting pipeline (game + viewmodel smoke test)

  The smoke scenario for the functional suite: it drives the game through an
  experiment and then reads the result back through the viewmodel layer,
  proving game + viewmodel are wired end to end without any front end.

  Scenario: A recipe discovered by experiment appears in the craft menu
    Given a new game
    And the player has 2 Vine in storage
    When the player experiments with 2 Vine
    Then storage contains 1 Cord
    And a "Cord" craft is offered in the craft menu
