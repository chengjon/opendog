# Config Incremental Editing Design

Date: 2026-05-06
Status: proposed
Scope: config UX hardening

## Goal

Strengthen `opendog config set-global` and `opendog config set-project` so operators can make small list edits safely without having to restate the full target list every time.

The target is intentionally narrow:

- keep the existing `--ignore-pattern` and `--process` flags as full overwrite operations
- add explicit incremental flags for append and removal
- support the same behavior in direct CLI mode and daemon-backed mode
- preserve current persisted config schema shape
- avoid any change to unrelated config fields, monitoring behavior, or report surfaces

This is configuration mutation hardening, not a broader config-system redesign.

## Capability Scope

Primary surfaces touched:

- `opendog config set-global`
- `opendog config set-project`
- daemon control requests for global and project config updates

Primary modules touched:

- `src/cli/config_commands.rs`
- `src/config.rs`
- `src/config/patching.rs`
- `src/core/project.rs`
- `src/control/protocol.rs`
- `src/control/request_handler.rs`
- `src/control/client/config_ops.rs`

## Current Problem

Today both config mutation commands treat repeated list flags as complete replacement:

- `--ignore-pattern ...`
- `--process ...`

That behavior is internally consistent, but it is easy to misuse operationally:

- adding one pattern requires restating the full current ignore list
- removing one process requires restating the full remaining process list
- project-level edits are especially error-prone when the project currently inherits global defaults
- CLI and daemon paths both share overwrite-only semantics, so there is no safer incremental path anywhere in the stack

The result is avoidable config drift caused by operators issuing a command that looks additive but is actually destructive.

## Design

### 1. Keep Existing Overwrite Flags Stable

The current flags remain valid and keep their meaning:

- `--ignore-pattern`
- `--process`

If either is present for a field, that field is treated as a full replacement for the target list after normalization.

This preserves backward compatibility for existing automation and existing documentation examples that intentionally rely on overwrite behavior.

### 2. Add Explicit Incremental Flags

Add four new flags:

- `--add-ignore-pattern`
- `--remove-ignore-pattern`
- `--add-process`
- `--remove-process`

These flags are available on both:

- `config set-global`
- `config set-project`

Semantics:

- `add` appends new values after normalization and de-duplication
- `remove` deletes exact normalized matches
- values that do not exist are ignored rather than treated as an error
- resulting lists keep stable order for surviving items

This makes additive and subtractive intent explicit instead of inferred.

### 3. Extend Patch Types Rather Than Hiding Merge Logic In The CLI

`ConfigPatch` and `ProjectConfigPatch` should gain incremental operation fields in addition to the existing overwrite fields.

Preferred shape:

- overwrite fields:
  - `ignore_patterns: Option<Vec<String>>`
  - `process_whitelist: Option<Vec<String>>`
- incremental fields:
  - `add_ignore_patterns: Vec<String>`
  - `remove_ignore_patterns: Vec<String>`
  - `add_process_whitelist: Vec<String>`
  - `remove_process_whitelist: Vec<String>`

`ProjectConfigPatch` continues to carry:

- `inherit_ignore_patterns: bool`
- `inherit_process_whitelist: bool`

The CLI should build these patch objects, but patch application must stay centralized in config/core logic so direct mode and daemon mode share one source of truth.

### 4. Project Inheritance Behavior Must Materialize Effective State Before Incremental Edits

User-approved rule:

- if a project field currently inherits global defaults and the operator uses `add/remove` for that field, OPENDOG should first derive the current effective list, then persist a project override containing the modified result

Example:

- global process whitelist = `["claude", "codex"]`
- project has no local process override
- operator runs `set-project --id demo --remove-process claude`

Result:

- project override becomes `["codex"]`
- inheritance for that field is no longer active

This is intentional. Incremental edits express a desire to diverge from the inherited baseline, so OPENDOG should make that divergence explicit and durable.

### 5. Conflict Rules Must Be Explicit

Within the same field:

- `--ignore-pattern` cannot be combined with `--add-ignore-pattern` or `--remove-ignore-pattern`
- `--process` cannot be combined with `--add-process` or `--remove-process`

For project config only:

- `--inherit-ignore-patterns` cannot be combined with any ignore-pattern overwrite or incremental flag
- `--inherit-process-whitelist` cannot be combined with any process overwrite or incremental flag

Across different fields, mixed operations are allowed. For example:

- overwrite ignore patterns while incrementally editing processes
- inherit process whitelist while overwriting ignore patterns

Conflict detection should happen as early as possible in CLI parsing, with core-layer validation kept as a defensive backstop.

### 6. Global And Project Application Rules

Global updates:

- start from current global defaults
- apply overwrite or add/remove operations per field
- save the final normalized `ProjectConfig`

Project updates:

- start from the stored project overrides plus current global defaults
- for overwrite, set the override list directly
- for inherit, clear the override field
- for add/remove on an overridden field, apply operations to the current override list
- for add/remove on an inherited field, first resolve from effective config, then save the modified list as an override

Normalization rules remain unchanged:

- trim whitespace
- drop empty values
- de-duplicate while preserving first occurrence order

### 7. Empty-Patch Protection Remains Required

No-op commands should still fail with a clear error.

That includes:

- no overwrite fields
- no add/remove values
- no inherit flag

This protection should remain in patch-level emptiness checks so daemon requests cannot bypass it.

## Implementation Shape

Primary ownership should remain split as follows:

- CLI layer
  - defines new flags
  - enforces argument conflicts where clap can express them clearly
  - constructs richer patch objects
- config patching layer
  - normalizes incremental inputs
  - applies overwrite and add/remove operations deterministically
  - exposes helper logic reusable by both global and project update flows
- project manager layer
  - resolves project effective config when incremental project operations need an inherited baseline
  - persists updated global config or project overrides
- control protocol layer
  - serializes new patch fields so daemon-backed updates behave the same as local updates

Storage format does not change. Persisted project records still store only `ProjectConfigOverrides`, and global config still stores only the final `ProjectConfig`.

## Test Strategy

Follow TDD: add failing tests first, then implement until they pass.

Primary coverage areas:

1. Global incremental edits
- add ignore pattern to existing defaults
- remove process from existing defaults
- de-duplicate repeated add inputs

2. Project incremental edits on existing overrides
- add to override list
- remove from override list
- preserve unrelated inherited field

3. Project incremental edits on inherited fields
- start with no override
- apply add/remove against effective global list
- confirm a concrete project override is persisted

4. Conflict validation
- reject overwrite plus add/remove for the same field
- reject inherit plus overwrite/add/remove for the same field

5. Daemon path parity
- send update through control request/response path
- confirm resulting effective config matches direct mode behavior
- confirm reload metadata still reflects actual changed fields

Likely test files:

- `src/config.rs`
- `src/config/patching.rs`
- `src/control/tests.rs`
- `tests/integration_test/storage_project_snapshot.rs`
- `tests/integration_test/daemon_process_cli.rs`

## Non-Goals

Do not:

- change existing overwrite semantics
- infer additive intent from existing flags
- add new config fields outside ignore/process lists
- change on-disk config schema shape
- redefine project inheritance behavior beyond the approved materialize-on-increment rule
- update operator-facing docs in this batch
