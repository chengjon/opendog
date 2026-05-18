## ADDED Requirements

### Requirement: MCP schema derives use a single schemars path
The system SHALL compile MCP parameter and DTO schema derives through one schema-derive path so the dependency graph does not carry duplicate `schemars` versions for the same contract surface.

#### Scenario: Duplicate schemars versions are eliminated
- **WHEN** the project dependency graph is inspected with `cargo tree -d`
- **THEN** the graph SHALL not report both `schemars v0.8.x` and `schemars v1.x` as active dependencies of the project surface

#### Scenario: Schema derives still compile after cleanup
- **WHEN** the project builds after the dependency cleanup
- **THEN** the MCP parameter and orphan DTO types SHALL still derive JSON schema successfully

### Requirement: Dependency hygiene is checked during release preparation
The system SHALL include a documented dependency hygiene check for duplicate contract-related dependencies before release or merge of dependency cleanup changes.

#### Scenario: Hygiene check is part of verification
- **WHEN** a dependency hygiene change is proposed or merged
- **THEN** the verification checklist SHALL include a duplicate-dependency review step

