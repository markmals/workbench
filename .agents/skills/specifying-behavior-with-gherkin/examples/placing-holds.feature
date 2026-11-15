Feature: Placing holds
  Library members place holds on titles that have no copy available to them.
  The library sets each returned copy aside for the member who has waited
  longest, and tells that member it is ready to collect.

  An active hold is waiting in a title's queue or has a copy ready for pickup.
  A title's queue orders its waiting holds by when they were placed.
  A ready copy is kept for seven days.
  How members are told is specified with notification preferences, not here.

  Rule: A member can have at most five active holds

    Scenario Outline: A member below the limit places a hold
      Given Ada already has <existing> active holds
      When Ada places a hold on "Kindred"
      Then Ada is in the queue for "Kindred"
      And Ada has <total> active holds in total

      Examples:
        | existing | total |
        | 0        | 1     |
        | 4        | 5     |

    Scenario: A member at the limit is refused
      Given Ada already has 5 active holds
      When Ada places a hold on "Kindred"
      Then the hold is refused because Ada has reached the limit of 5 active holds
      And Ada is not in the queue for "Kindred"
      And Ada has 5 active holds in total

  Rule: Copies go to the queue in the order holds were placed

    Background:
      Given the queue for "Kindred" is:
        | member | placed at        |
        | Grace  | 2026-03-01 09:00 |
        | Ada    | 2026-03-02 14:30 |

    Scenario: A returned copy goes to the longest-waiting member
      When a copy of "Kindred" is returned
      Then the copy is ready for pickup by Grace
      And Grace is told that "Kindred" is ready for pickup
      And Ada is first in the queue for "Kindred"

    Scenario: An uncollected copy passes to the next member
      Given a copy of "Kindred" has been ready for pickup by Grace for 7 days
      When the pickup period ends
      Then Grace's hold on "Kindred" ends
      And the copy is ready for pickup by Ada
