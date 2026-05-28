# orphan-module-boundary Specification

## Purpose
Define the module boundary for orphan detection after splitting implementation internals. These requirements preserve the public MCP-facing API while assigning scanner contracts and candidate collection to focused submodules.
## Requirements
### Requirement: Orphan detection internals preserve their public API during module split
The system SHALL be able to move orphan detection internals into focused submodules while preserving the public API used by MCP handlers through re-exports or equivalent compatibility shims.

#### Scenario: MCP callers keep the same imports
- **WHEN** orphan internals are split into `src/core/orphan/` submodules
- **THEN** existing MCP callers SHALL continue to compile without changing their public imports

#### Scenario: Public behavior remains unchanged
- **WHEN** the module split is complete
- **THEN** the orphan unit tests, MCP payload contract tests, and MCP session integration test SHALL still pass with no behavioral changes

### Requirement: Orphan responsibilities are assigned to named submodules
The system SHALL separate scanner-health validation, candidate collection, built-in evidence scanning, and deletion-plan verification into named submodules before phase 2 persistence or external scanner execution is added.

#### Scenario: Scanner contract logic is isolated
- **WHEN** scanner-health validation is implemented
- **THEN** it SHALL live in a named scanner-contract-oriented module

#### Scenario: Candidate collection logic is isolated
- **WHEN** built-in candidate collection or evidence scanning changes
- **THEN** that logic SHALL live in a named orphan submodule instead of growing the facade module
