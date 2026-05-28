# mcp-tool-inventory Specification

## Purpose
Define the MCP tool inventory as the source of truth for public tool metadata and registration validation. These requirements make tool-surface drift visible in code review and regression tests.
## Requirements
### Requirement: MCP tool inventory is the source of truth for tool metadata
The system SHALL maintain a single inventory of MCP tools that enumerates each tool's name, contract ID, params type, payload builder, handler module, and test owner.

#### Scenario: Inventory entry covers a tool completely
- **WHEN** a new MCP tool is added to the inventory
- **THEN** the entry SHALL identify the tool name, contract ID, params type, payload builder, handler module, and test owner

#### Scenario: Existing tools remain in inventory
- **WHEN** the inventory is reviewed
- **THEN** every currently registered MCP tool SHALL appear in the inventory

### Requirement: Tool registration is validated against the inventory
The system SHALL validate the registered MCP tool surface against the inventory so missing registrations or stale metadata fail fast in tests.

#### Scenario: Missing registration fails validation
- **WHEN** a tool exists in the inventory but is missing from runtime registration
- **THEN** the tool-surface validation SHALL fail

#### Scenario: Inventory drift is visible in tests
- **WHEN** a tool name, contract ID, or payload builder drifts from the inventory
- **THEN** the corresponding tool-surface or payload contract test SHALL fail
