# Orphan Detection Rust Framework Design

## Goal

Add an OpenDog-native orphan detection framework without turning OpenDog into a Python,
TypeScript, or framework-specific semantic analyzer.

The Rust side owns:

- candidate discovery from registered project files
- language-neutral scanners
- normalized evidence ingestion
- evidence aggregation
- deletion-safety classification
- MCP payload contracts
- optional persisted scan history

Language-specific scanners remain external producers. They can be shipped later, but
the Rust contract must be useful before any external scanner exists.

## Non-Goals

OpenDog must not directly implement these in Rust:

- Python import graph semantics
- FastAPI route registration semantics
- pytest collection semantics
- TypeScript AST consumer analysis
- telemetry backends such as nginx, OpenTelemetry, Prometheus, or APM vendors
- code deletion or automatic cleanup commands

OpenDog can execute configured scanner commands in a later phase, but execution is
only orchestration. The scanner's language semantics stay outside the OpenDog core.

## Design Principle

The core contract is normalized evidence, not scanner-specific output.

Each scanner, whether Rust-native or external, reports evidence signals. The
classifier sees only normalized signals and scanner health. It does not need to know
how LibCST, FastAPI, pytest, TypeScript, or telemetry systems produced those signals.

This keeps the Rust implementation small and lets scanner quality improve
independently.

## Repository Fit

The implementation should follow the existing OpenDog split:

- `src/core/orphan.rs` for the Phase 1 model, scanners, aggregation, and classification
- `src/mcp/orphan_handlers.rs` for thin MCP request handling
- `src/mcp/orphan_payload.rs` or `src/mcp/payloads.rs` for payload construction
- `src/storage/queries/orphan_detection.rs` for persisted scan history, if enabled
- `src/control/*` only if orphan scan results are daemon-backed or persisted

Do not place the main implementation under `src/mcp/orphan_detection/`. MCP modules
should stay thin and should not become the business-logic owner.

Phase 1 should start as a single `src/core/orphan.rs` file because current core
modules are flat. Split to a directory-based core module only after the module grows
large enough that separate `model`, `classifier`, and `scanner` files reduce real
complexity.

## Core Model

### Subject

An orphan candidate can refer to a file, module, route, URL path, package entrypoint,
or another named resource.

Required fields:

- `subject_kind`: `file | module | route | url | command | unknown`
- `subject`: stable user-facing identifier
- `path`: optional repository-relative file path
- `display_name`: optional shorter label

### Evidence Signal

All scanners normalize into this shape:

- `source`: scanner name, such as `entrypoint_scanner`, `docs_ownership_gate`, `python_import_graph`
- `source_kind`: `rust_internal | external_report | external_command | manual`
- `signal_kind`: `incoming_ref | outgoing_ref | runtime_route | openapi_path | test_coverage | entrypoint | frontend_consumer | docs_owner | telemetry | dynamic_import_risk | scanner_warning`
- `polarity`: `supports_used | supports_unused | veto | informational`
- `confidence`: floating point value from `0.0` to `1.0`
- `observed_at`: optional unix timestamp
- `subject`: normalized subject reference
- `detail`: scanner-specific JSON detail

The Rust classifier must treat unknown signal kinds as `informational`, not as proof.

### Scanner Health

Each expected scanner reports a health state:

- `passed`
- `passed_with_warnings`
- `skipped`
- `failed`
- `unavailable`

`skipped`, `failed`, and `unavailable` are not equivalent to negative evidence. They
are uncertainty and should normally cap the result at `review_required`.

## Classification

The output classification is intentionally not binary:

- `remove_candidate`
- `review_required`
- `blocked`

## Required Scanner Contract

`required_scanners` is load-bearing because `remove_candidate` is allowed only when
all required scanner health checks are acceptable.

When `required_scanners` is omitted, OpenDog derives the requirement set from the
project and subjects:

- always required: `candidate_collector`, `entrypoint_scanner`, `docs_ownership_gate`
- required when a frontend workspace or URL subject is detected: `frontend_literal_scanner`
- required for Python module or Python file subjects in Python projects: `python_import_graph`
- required for FastAPI route or URL subjects in FastAPI-like projects: `fastapi_route_auditor`
  and `openapi_contract`

Phase 1 can derive external scanner requirements even though it does not run those
scanners. Missing required external reports cap affected candidates at
`review_required`.

When the caller supplies `required_scanners`, the list is additive in Phase 1. It can
require extra scanners but cannot remove derived safety requirements. This avoids a
caller accidentally promoting a candidate by omitting a scanner.

Validation rules:

- omitted `required_scanners` means use derived defaults
- an empty `required_scanners` array is invalid input
- each scanner name must be a known Rust-internal scanner or match a scanner name in
  `external_reports`
- duplicate scanner names are ignored after validation
- `include_internal_scanners = false` is allowed only when equivalent internal
  scanner reports are supplied through `external_reports`; otherwise affected
  candidates are `review_required`

### Blocked

A candidate is `blocked` when any high-priority veto exists:

- runtime route signal says the candidate is registered
- OpenAPI path signal says the candidate remains in the public contract
- entrypoint signal references the candidate
- frontend consumer signal references the candidate
- telemetry signal shows recent use
- docs or ownership signal marks the subject as public, owned, protected, or do-not-delete
- dynamic import risk applies to the subject and no stronger runtime evidence clears it

### Review Required

A candidate is `review_required` when the evidence is incomplete or stale:

- one or more required scanners are `failed`, `skipped`, or `unavailable`
- freshness threshold is exceeded
- only lightweight text scanning ran
- subject-to-file mapping is ambiguous
- scanner warnings affect this subject
- confidence is below the remove threshold

### Remove Candidate

A candidate may be `remove_candidate` only when:

- all required scanner health checks are acceptable
- no veto signal exists
- no used signal exists above the configured confidence threshold
- enough applicable scanners produced negative evidence
- evidence freshness is within threshold
- the response includes recommended verification commands

This classification means "safe candidate for human-reviewed deletion planning", not
"delete this file automatically".

## Confidence

Use the current assessment's formula as a scoring aid, not as the primary safety
decision:

```text
confidence = base_confidence * signal_density * freshness_factor
```

Safety gates run before confidence promotion:

- any veto makes the result `blocked`
- missing required scanner health caps the result at `review_required`
- stale evidence caps the result at `review_required`
- unknown scanner output is informational only

Confidence should explain ranking within a class. It should not override vetoes.

## Rust-Internal Scanners

### Candidate Collector

Inputs:

- registered project root
- effective project config
- current snapshot rows, if present
- filesystem walk as fallback

Behavior:

- reuse OpenDog ignore rules
- emit file subjects for source-like files
- classify paths with existing file classification rules
- exclude infrastructure and backup paths by default
- exclude project documentation paths by explicit path-prefix rules when the subject
  mode is source-file cleanup
- do not claim generated-file exclusion in Phase 1; `FilePathClassification` has no
  `Generated` variant yet, so generated-file detection is a follow-up unless that
  enum is extended first

### Entrypoint Scanner

Scan language-neutral operational files:

- `Dockerfile`
- `docker-compose*.yml`
- `Procfile`
- `pm2*.json`
- `.github/workflows/**`
- `scripts/**`
- `Makefile`
- `systemd` unit files

Detect textual references to:

- `uvicorn module:app`
- `gunicorn module:app`
- `python -m module`
- shell commands invoking known modules
- explicit file paths

Findings become `entrypoint` signals. Strong matches are `veto`; fuzzy matches are
`informational` or `supports_used`.

### Docs Ownership Gate

Scan repository governance files:

- `OWNERS`
- `CODEOWNERS`
- `architecture/STANDARDS.md`
- `openspec/**`
- `.planning/**`
- `docs/**`

Signals:

- explicit ownership or protected API produces `docs_owner` veto
- explicit cleanup allowance produces `supports_unused`
- ambiguous references produce `informational`

### Frontend Literal Scanner

Phase 1 only performs lightweight string scanning for URL literals and API path
segments. TypeScript AST analysis is external and later.

Strong literal URL matches produce `frontend_consumer` veto. Fuzzy fragment matches
produce `review_required` evidence.

## External Report Ingestion

Phase 1 accepts external reports inline through MCP. It does not execute external
scanner commands.

External scanner reports must include:

- scanner name
- version
- root path or project identity
- started/finished timestamp
- health state
- normalized evidence signals
- warnings
- errors

Scanner-specific raw details may be embedded under `detail`, but OpenDog should not
require those details to classify candidates.

## MCP Surface

### `scan_orphans`

Required:

- `id`

Optional:

- `subjects`
- `external_reports`
- `include_internal_scanners`
- `required_scanners`
- `max_age_secs`
- `limit`
- `include_evidence`

Response:

- `schema_version`
- `project_id`
- `status`
- `scan_run_id`, if persisted
- `scanner_health`
- `summary`
- `candidates`
- `warnings`
- `recommended_next_actions`

Error responses:

- project not found uses the existing project error contract
- malformed input uses the existing validation error contract
- malformed `external_reports` or unknown scanner names use the validation error
  contract
- scanner health problems are returned in the normal payload as `review_required`
  or `blocked`, not as a transport error, unless the request cannot be classified at
  all

### `verify_deletion_plan`

Required:

- `id`
- `targets`

Optional:

- `scan_run_id`
- `external_reports`
- `required_project_verification_commands`
- `max_age_secs`

Response:

- `schema_version`
- `project_id`
- `status`
- `safe_to_plan_deletion`: boolean
- `blocked_targets`
- `review_required_targets`
- `remove_candidates`
- `required_project_verification_commands`
- `evidence_gaps`

`safe_to_plan_deletion` does not mean OpenDog will delete files. It means the evidence
is strong enough for a human or agent to draft a deletion patch.

`required_project_verification_commands` refers to the existing test/lint/build style
verification commands that should be run through OpenDog's existing verification
surface before a human-approved deletion patch is produced. It is not a new
deletion-safety command category.

Error responses:

- project not found uses the existing project error contract
- malformed targets, unknown scanner names, or unusable `scan_run_id` values use the
  existing validation error contract
- evidence gaps are normally returned in `evidence_gaps`; they are not top-level
  errors unless no target can be evaluated

## Persistence

Persistence is useful for traceability but should not block the first classifier unit
tests.

When these tables are added, bump the project database `SCHEMA_VERSION` and treat
stored confidence as point-in-time output. The persisted classification must include
the classifier and confidence formula versions so old rows are not mistaken for
current safety decisions after the scoring logic changes.

Recommended tables:

```sql
CREATE TABLE orphan_scan_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scanned_at TEXT NOT NULL,
    status TEXT NOT NULL,
    root_path TEXT NOT NULL,
    scanner_summary_json TEXT NOT NULL,
    warnings_json TEXT NOT NULL DEFAULT '[]',
    errors_json TEXT NOT NULL DEFAULT '[]'
);

CREATE TABLE orphan_candidates (
    scan_run_id INTEGER NOT NULL,
    subject_kind TEXT NOT NULL,
    subject TEXT NOT NULL,
    path TEXT,
    classification TEXT NOT NULL,
    classification_version TEXT NOT NULL,
    confidence REAL NOT NULL,
    confidence_formula_version TEXT NOT NULL,
    reasons_json TEXT NOT NULL,
    vetoes_json TEXT NOT NULL,
    evidence_json TEXT NOT NULL,
    PRIMARY KEY (scan_run_id, subject_kind, subject)
);
```

Start with JSON evidence blobs. Split evidence into a relational table only after
query needs become clear.

## Implementation Phases

### Phase 1: Rust Framework And Inline Evidence

- add core orphan detection model and classifier
- add candidate collector
- add entrypoint scanner
- add docs ownership gate
- add frontend literal scanner
- add MCP `scan_orphans`
- add MCP `verify_deletion_plan`
- accept inline normalized external reports
- add focused unit and MCP payload tests

No external command execution in this phase.

### Phase 2: Scanner Command Orchestration

- add configured external scanner command execution
- capture stdout/stderr tails
- parse normalized JSON reports from stdout
- record scanner health
- add timeout and failure handling
- store scan history

This should reuse the existing verification-command orchestration style but keep
scanner command kinds separate from test/lint/build verification.

### Phase 3: Language-Specific Scanner Packages

- Python import graph scanner
- FastAPI route auditor
- OpenAPI exporter/diff producer
- pytest collect mapper
- TypeScript API consumer scanner
- telemetry adapter accepting exported traffic summaries

These can live outside the Rust crate or under a clearly separated external tooling
directory. They should be versioned against the normalized evidence contract.

## Test Plan

Unit tests:

- veto wins over confidence
- failed required scanner caps at `review_required`
- stale evidence caps at `review_required`
- all-clear evidence can classify as `remove_candidate`
- unknown signal kinds stay informational
- missing derived required scanners cap at `review_required`
- empty or unknown `required_scanners` values fail validation

Scanner fixture tests:

- Dockerfile and compose references
- PM2 and shell command references
- docs ownership vetoes
- frontend URL literal vetoes
- fuzzy URL fragments require review

MCP tests:

- tool surface exposes `scan_orphans` and `verify_deletion_plan`
- payloads include schema version and project id
- inline external report fixture affects classification
- missing project returns the existing project error contract
- malformed external reports and invalid scanner names return the validation error
  contract

Integration tests:

- register temp project
- create representative source, Dockerfile, docs, and frontend files
- call `scan_orphans`
- assert blocked/review/remove classifications

## Open Questions

- Should persisted orphan scan history be Phase 1 or Phase 2?
- Should CLI expose these surfaces immediately, or should the first release be MCP-only?
- Should per-project protected path globs be added as a new config key distinct from
  existing ignore patterns, so protected files remain visible but produce deletion
  vetoes?

## Recommendation

Ship Phase 1 as MCP-only Rust framework with inline external evidence ingestion. This
proves the contract, classifier, and safety semantics before committing to scanner
execution or language-specific packages.
