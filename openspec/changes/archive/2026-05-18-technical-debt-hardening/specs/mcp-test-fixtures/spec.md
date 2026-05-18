## ADDED Requirements

### Requirement: Large MCP tests use reusable domain fixture builders
The system SHALL provide reusable fixture builders for repeated MCP payload and guidance scenarios so the largest tests describe domain facts instead of repeating inline JSON setup.

#### Scenario: Repeated setup is centralized
- **WHEN** a test cluster repeatedly needs the same project, risk, verification, or recommendation setup
- **THEN** the cluster SHALL use a shared fixture builder or helper

#### Scenario: Assertions stay behavior-focused
- **WHEN** a test uses a fixture builder
- **THEN** the test SHALL still assert the relevant schema fields, command strings, and failure states explicitly

### Requirement: Contract tests remain readable after fixture extraction
The system SHALL keep payload contract and guidance tests readable by naming helpers after business facts instead of raw JSON fragments.

#### Scenario: Helper names describe domain state
- **WHEN** a test helper is added
- **THEN** its name SHALL describe the domain state it creates

#### Scenario: Payload contracts still verify key surface fields
- **WHEN** a payload contract test uses shared fixtures
- **THEN** it SHALL continue to verify representative command strings and schema fields

