Feature: Applying database migrations
  Developers run "migrate up" to bring a database schema to the version
  their code expects. The command applies migration files from the
  project's migrations directory and records each applied file in the
  database's migration history.

  A migration is identified by its file name. A pending migration is in
  the directory but not in the history. Every rule applies to each
  supported database unless the rule says otherwise.

  Background:
    Given the migrations directory contains:
      | file                   |
      | 0001_create_users.sql  |
      | 0002_add_email.sql     |
      | 0003_create_orders.sql |

  Rule: Pending migrations are applied once each, in file name order

    Scenario: Only pending migrations are applied
      Given the history lists "0001_create_users.sql"
      When the developer runs "migrate up"
      Then the command succeeds, reporting 2 applied migrations
      And the history lists, in order:
        | file                   |
        | 0001_create_users.sql  |
        | 0002_add_email.sql     |
        | 0003_create_orders.sql |

    Scenario: Nothing is pending
      Given the history lists every migration in the directory
      When the developer runs "migrate up"
      Then the command succeeds, reporting no pending migrations
      And the history is unchanged

  Rule: A failing migration stops the run and leaves later migrations pending

    Scenario: The second of three pending migrations fails
      Given the history is empty
      And "0002_add_email.sql" fails with "column email already exists"
      When the developer runs "migrate up"
      Then the command fails, reporting "0002_add_email.sql" and "column email already exists"
      And the history lists only "0001_create_users.sql"
      And "0002_add_email.sql" and "0003_create_orders.sql" are still pending

  @transactional-ddl
  Rule: A failed migration leaves no partial schema changes
    Applies where the database can roll back schema changes in a transaction,
    such as PostgreSQL and SQLite. MySQL commits each schema change as it
    runs, so its bindings exclude this rule and report it as not applicable.

    Scenario: A migration fails after its first statement
      Given the history lists "0001_create_users.sql"
      And the second statement of "0002_add_email.sql" fails
      When the developer runs "migrate up"
      Then the command fails, reporting "0002_add_email.sql"
      And the schema is the same as before the run
      And "0002_add_email.sql" is still pending
