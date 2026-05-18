## ADDED Requirements

### Requirement: Registry and project databases are migration-aware
The system SHALL apply pending schema migrations when opening a registry or project database, and it SHALL keep the on-disk schema version aligned with `SCHEMA_VERSION` via `PRAGMA user_version`.

#### Scenario: Fresh database open sets schema version
- **WHEN** the system opens a new registry or project database
- **THEN** the database SHALL be initialized to the current schema version
- **AND** `PRAGMA user_version` SHALL equal `SCHEMA_VERSION` after open

#### Scenario: Older fixture migrates forward
- **WHEN** the system opens a fixture database created at a previous supported schema version
- **THEN** the system SHALL apply the pending migration steps before returning the handle
- **AND** the migrated database SHALL preserve representative snapshot, stats, and verification data

### Requirement: Storage migration behavior is regression-tested
The system SHALL include tests that prove the migration path and the post-open schema version are correct for at least one older fixture database.

#### Scenario: Migration regression passes
- **WHEN** the migration test opens an older fixture database
- **THEN** the test SHALL verify that the database opens successfully
- **AND** the test SHALL verify that the resulting schema version matches the current `SCHEMA_VERSION`

