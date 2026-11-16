# Copy to specs/features/<capability>.feature, or to the project's existing
# feature location. Describe one capability per file. Replace each <...>
# placeholder, and delete the comments and any section this capability does
# not need. Guidance: the specifying-behavior-with-gherkin skill.
Feature: <Capability, named in its users' words>
  <Who uses this capability and what it lets them do.>

  <Terms the rules rely on, and what this feature leaves to other owners.>

  # Optional: Given steps that every scenario below needs. A Rule can have
  # its own Background for steps that only its scenarios need.
  Background:
    Given <context every scenario shares>

  Rule: <A business rule, stated so that it is either true or false>

    Scenario: <A concrete case that illustrates the rule>
      Given <the relevant state, in domain terms>
      When <the event or action>
      Then <the observable outcome>

    Scenario: <A case the rule refuses>
      Given <state that triggers the refusal>
      When <the event or action>
      Then <the observable refusal and its reason>
      And <what stays unchanged>

  Rule: <Another business rule>

    # An outline runs once per Examples row. <input> and <expected> are
    # parameters: rename them to match the column headers below.
    Scenario Outline: <The values at which the rule changes its outcome>
      Given <context>
      When <action> with <input>
      Then <outcome> is <expected>

      Examples:
        | input   | expected |
        | <value> | <value>  |
        | <value> | <value>  |

    # A data table attaches rows to the step above it.
    Scenario: <A case that needs several records>
      Given <records>:
        | <column> | <column> |
        | <value>  | <value>  |
      When <the event or action>
      Then <the observable outcome>
