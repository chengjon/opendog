# runtime-error-boundaries Specification

## Purpose
TBD - created by archiving change technical-debt-hardening. Update Purpose after archive.
## Requirements
### Requirement: MCP and control startup failures are structured errors
The system SHALL return structured errors for startup and mutex-lock failures in MCP/control code paths instead of panicking inside the application logic.

#### Scenario: Poisoned mutex becomes an OpenDog error
- **WHEN** a control or MCP helper attempts to lock a poisoned mutex
- **THEN** the helper SHALL return a structured error
- **AND** the process SHALL not panic inside the helper

#### Scenario: Startup failure is reported once at the boundary
- **WHEN** MCP startup cannot create its runtime or server
- **THEN** the process boundary SHALL log the failure and exit non-zero
- **AND** the lower-level helper SHALL expose the failure as a `Result`

### Requirement: Production payload serialization does not panic on invariant failure
The system SHALL avoid production `expect`/`unwrap` calls in MCP payload and decision helpers when converting serialization invariants into payloads.

#### Scenario: Payload conversion failure is surfaced as a structured error
- **WHEN** a payload helper cannot serialize a documented invariant value
- **THEN** the helper SHALL return a structured error path rather than panicking

#### Scenario: Verification post-insert lookup remains recoverable
- **WHEN** the verification layer cannot find the run immediately after insertion
- **THEN** the system SHALL surface a domain error that callers can handle

