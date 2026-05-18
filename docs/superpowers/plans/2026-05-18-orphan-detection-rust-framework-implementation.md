# Orphan Detection Rust Framework Implementation Plan

## Source Spec

- Design: `docs/superpowers/specs/2026-05-18-orphan-detection-rust-framework-design.md`
- Review: `docs/superpowers/specs/2026-05-18-orphan-detection-rust-framework-design-review.md`

## Scope

Implement Phase 1 only:

- Rust-only core model and classifier
- Rust-internal candidate, entrypoint, docs, and frontend-literal scanners
- inline normalized external evidence ingestion
- MCP `scan_orphans`
- MCP `verify_deletion_plan`
- no database persistence
- no daemon protocol changes
- no external command execution
- no CLI surface

`scan_run_id` should be omitted or `null` in Phase 1 payloads because no scan history is
persisted.

## Pre-Flight

Run:

```bash
git -C /opt/claude/opendog status --short
```

Expected:

- unrelated untracked files may exist
- do not remove or revert them

## Task 1: Add Core Module Skeleton And Unit Tests

Files:

- Create: `src/core/orphan.rs`
- Modify: `src/core/mod.rs`

Add the module export:

```rust
pub mod orphan;
```

Create `src/core/orphan.rs` with data types and failing tests first. The first version
may use minimal function bodies that compile only after Task 2 fills them in; keep the
tests exact.

Required public types:

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OrphanSubjectKind {
    File,
    Module,
    Route,
    Url,
    Command,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OrphanSubject {
    pub subject_kind: OrphanSubjectKind,
    pub subject: String,
    pub path: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EvidencePolarity {
    SupportsUsed,
    SupportsUnused,
    Veto,
    Informational,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScannerHealth {
    Passed,
    PassedWithWarnings,
    Skipped,
    Failed,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OrphanClassification {
    RemoveCandidate,
    ReviewRequired,
    Blocked,
}
```

Required tests:

```rust
#[test]
fn veto_signal_blocks_candidate() {
    let subject = file_subject("src/api/old.py");
    let result = classify_subject(
        &subject,
        vec![scanner_health("entrypoint_scanner", ScannerHealth::Passed)],
        vec![signal(&subject, "entrypoint", EvidencePolarity::Veto, 0.95)],
        &ClassificationOptions::default(),
    )
    .unwrap();

    assert_eq!(result.classification, OrphanClassification::Blocked);
    assert!(result.vetoes.iter().any(|item| item.contains("entrypoint")));
}

#[test]
fn missing_required_scanner_caps_at_review_required() {
    let subject = file_subject("src/api/old.py");
    let result = classify_subject(
        &subject,
        vec![scanner_health("candidate_collector", ScannerHealth::Passed)],
        vec![signal(&subject, "candidate_collector", EvidencePolarity::SupportsUnused, 0.95)],
        &ClassificationOptions {
            required_scanners: vec![
                "candidate_collector".to_string(),
                "entrypoint_scanner".to_string(),
            ],
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(result.classification, OrphanClassification::ReviewRequired);
    assert!(result
        .reasons
        .iter()
        .any(|item| item.contains("entrypoint_scanner")));
}

#[test]
fn all_required_scanners_with_unused_evidence_can_remove_candidate() {
    let subject = file_subject("src/api/old.py");
    let result = classify_subject(
        &subject,
        vec![
            scanner_health("candidate_collector", ScannerHealth::Passed),
            scanner_health("entrypoint_scanner", ScannerHealth::Passed),
            scanner_health("docs_ownership_gate", ScannerHealth::Passed),
        ],
        vec![
            signal(&subject, "candidate_collector", EvidencePolarity::SupportsUnused, 0.95),
            signal(&subject, "entrypoint_scanner", EvidencePolarity::SupportsUnused, 0.90),
            signal(&subject, "docs_ownership_gate", EvidencePolarity::SupportsUnused, 0.85),
        ],
        &ClassificationOptions::default(),
    )
    .unwrap();

    assert_eq!(result.classification, OrphanClassification::RemoveCandidate);
}

#[test]
fn unknown_signal_kind_is_informational() {
    let subject = file_subject("src/api/old.py");
    let mut evidence = signal(&subject, "custom_scanner", EvidencePolarity::SupportsUsed, 1.0);
    evidence.signal_kind = "unknown_future_signal".to_string();

    let result = classify_subject(
        &subject,
        vec![
            scanner_health("candidate_collector", ScannerHealth::Passed),
            scanner_health("entrypoint_scanner", ScannerHealth::Passed),
            scanner_health("docs_ownership_gate", ScannerHealth::Passed),
        ],
        vec![
            signal(&subject, "candidate_collector", EvidencePolarity::SupportsUnused, 0.95),
            signal(&subject, "entrypoint_scanner", EvidencePolarity::SupportsUnused, 0.90),
            signal(&subject, "docs_ownership_gate", EvidencePolarity::SupportsUnused, 0.85),
            evidence,
        ],
        &ClassificationOptions::default(),
    )
    .unwrap();

    assert_eq!(result.classification, OrphanClassification::RemoveCandidate);
}

#[test]
fn empty_required_scanners_is_invalid() {
    let error = validate_required_scanners(Some(&[]), &[]).unwrap_err();
    assert!(error.to_string().contains("required_scanners cannot be empty"));
}
```

Verification:

```bash
cargo test orphan:: --lib
```

Expected before Task 2:

- tests fail or the module does not compile because implementation is incomplete

## Task 2: Implement Classifier And Rust-Internal Scanners

Files:

- Modify: `src/core/orphan.rs`

Implement:

- `ClassificationOptions`
- `EvidenceSignal`
- `ScannerHealthEntry`
- `ClassifiedOrphanCandidate`
- `ScanOrphansInput`
- `ScanOrphansResult`
- `DeletionPlanInput`
- `DeletionPlanVerification`
- `validate_required_scanners`
- `derive_required_scanners`
- `classify_subject`
- `scan_project_orphans`
- `verify_deletion_plan`

Core rules:

```rust
const KNOWN_SIGNAL_KINDS: &[&str] = &[
    "incoming_ref",
    "outgoing_ref",
    "runtime_route",
    "openapi_path",
    "test_coverage",
    "entrypoint",
    "frontend_consumer",
    "docs_owner",
    "telemetry",
    "dynamic_import_risk",
    "scanner_warning",
    "candidate_collector",
];

const DEFAULT_REQUIRED_SCANNERS: &[&str] = &[
    "candidate_collector",
    "entrypoint_scanner",
    "docs_ownership_gate",
];
```

Classification order:

1. Unknown signal kinds become informational.
2. Any `EvidencePolarity::Veto` blocks.
3. Any known `SupportsUsed` signal at or above `used_signal_threshold` blocks.
4. Missing, failed, skipped, or unavailable required scanner health makes
   `review_required`.
5. Stale evidence makes `review_required`.
6. If there is at least one `SupportsUnused` signal and no blocker, classify as
   `remove_candidate`.
7. Otherwise classify as `review_required`.

Internal scanner behavior:

- candidate collector walks the project with `walkdir::WalkDir`
- ignore paths through `crate::config::should_ignore_path`
- classify paths with `crate::core::file_classification::classify_file_path`
- exclude `Infrastructure` and `Backup`
- exclude docs-like paths for source cleanup:
  - `docs/`
  - `.planning/`
  - `README.md`
  - files ending in `.md`
- entrypoint scanner reads only small operational files and scripts
- docs ownership gate scans governance/doc files for direct candidate path or subject
  mentions
- frontend literal scanner scans `web/`, `frontend/`, `src/`, and `app/` source files
  for URL/path literals matching URL subjects

Use only existing dependencies. Do not add `regex`; use `str::contains`, token
splitting, and path matching.

Verification:

```bash
cargo test orphan:: --lib
```

Expected:

- all `orphan::` tests pass

## Task 3: Add MCP Params, Payloads, And Handlers

Files:

- Modify: `src/contracts.rs`
- Modify: `src/mcp/params.rs`
- Create: `src/mcp/orphan_handlers.rs`
- Create: `src/mcp/orphan_payload.rs`
- Modify: `src/mcp/mod.rs`

Add contract constants:

```rust
pub const MCP_ORPHAN_SCAN_V1: &str = "opendog.mcp.orphan-scan.v1";
pub const MCP_ORPHAN_DELETION_PLAN_V1: &str = "opendog.mcp.orphan-deletion-plan.v1";
```

Add params:

```rust
#[derive(Deserialize, schemars::JsonSchema)]
pub struct ScanOrphansParams {
    pub id: String,
    pub subjects: Option<Vec<crate::core::orphan::OrphanSubject>>,
    pub external_reports: Option<Vec<crate::core::orphan::ExternalScannerReport>>,
    pub include_internal_scanners: Option<bool>,
    pub required_scanners: Option<Vec<String>>,
    pub max_age_secs: Option<u64>,
    pub limit: Option<usize>,
    pub include_evidence: Option<bool>,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub struct VerifyDeletionPlanParams {
    pub id: String,
    pub targets: Vec<crate::core::orphan::OrphanSubject>,
    pub external_reports: Option<Vec<crate::core::orphan::ExternalScannerReport>>,
    pub required_project_verification_commands: Option<Vec<String>>,
    pub max_age_secs: Option<u64>,
}
```

If `schemars::JsonSchema` derive fails for core orphan types, derive
`schemars::JsonSchema` on those core types as part of this task.

Payload functions:

```rust
pub(super) fn orphan_scan_payload(project_id: &str, result: &ScanOrphansResult) -> Value
pub(super) fn orphan_deletion_plan_payload(project_id: &str, result: &DeletionPlanVerification) -> Value
```

Handlers:

```rust
pub(super) fn handle_scan_orphans(
    server: &OpenDogServer,
    params: ScanOrphansParams,
) -> Json<Value>

pub(super) fn handle_verify_deletion_plan(
    server: &OpenDogServer,
    params: VerifyDeletionPlanParams,
) -> Json<Value>
```

Handler rules:

- get `ProjectInfo` through `server.get_project(&id)`
- resolve effective config by locking `server.inner` and calling
  `project_manager().effective_project_config(&id)`
- project-not-found uses `error_json_for(MCP_ORPHAN_SCAN_V1, Some(&id), &e)`
- invalid scanner input uses `OpenDogError::InvalidInput`
- scanner health gaps are payload data, not top-level errors

Verification:

```bash
cargo test mcp::tests::payload_contracts --lib
```

Expected:

- existing payload contract tests still pass

## Task 4: Register MCP Tools And Add Surface Tests

Files:

- Modify: `src/mcp/mod.rs`
- Modify: `src/mcp/tests/tool_surface.rs`

Add module imports:

```rust
mod orphan_handlers;
mod orphan_payload;
```

Export params:

```rust
ScanOrphansParams, VerifyDeletionPlanParams
```

Register tools:

```rust
#[tool(
    name = "scan_orphans",
    description = "Classify orphan cleanup candidates for one project using Rust-internal scanners and optional normalized external scanner reports. Required param: id. Optional params: subjects, external_reports, include_internal_scanners, required_scanners, max_age_secs, limit, include_evidence."
)]
fn scan_orphans(&self, Parameters(params): Parameters<ScanOrphansParams>) -> ToolResult {
    structured_tool_output(handle_scan_orphans(self, params))
}

#[tool(
    name = "verify_deletion_plan",
    description = "Verify whether proposed deletion targets have enough orphan-detection evidence for a human-reviewed deletion plan. Required params: id, targets. Optional params: external_reports, required_project_verification_commands, max_age_secs."
)]
fn verify_deletion_plan(
    &self,
    Parameters(params): Parameters<VerifyDeletionPlanParams>,
) -> ToolResult {
    structured_tool_output(handle_verify_deletion_plan(self, params))
}
```

Tool surface tests:

```rust
#[test]
fn orphan_detection_tools_are_exposed() {
    let mcp_router_source = include_str!("../mod.rs");
    assert!(mcp_router_source.contains("name = \"scan_orphans\""));
    assert!(mcp_router_source.contains("name = \"verify_deletion_plan\""));
}
```

Verification:

```bash
cargo test mcp::tests::tool_surface --lib
```

Expected:

- tool surface tests pass

## Task 5: Add Payload And Handler Tests

Files:

- Create: `src/mcp/tests/payload_contracts/orphan_payloads.rs`
- Modify: `src/mcp/tests/payload_contracts.rs`
- Create: `src/mcp/tests/orphan_handlers.rs`
- Modify: `src/mcp/tests.rs`

Payload contract test assertions:

- `schema_version == MCP_ORPHAN_SCAN_V1`
- `project_id` is present
- `status` is present
- `scan_run_id` is absent or null
- `scanner_health` is an array
- `candidates` is an array

Handler-style tests should exercise core functions directly when an MCP server fixture
would require too much setup:

```rust
#[test]
fn inline_external_veto_blocks_candidate() {
    let subject = file_subject("src/api/old.py");
    let report = external_report(
        "fastapi_route_auditor",
        ScannerHealth::Passed,
        vec![signal(&subject, "runtime_route", EvidencePolarity::Veto, 0.99)],
    );

    let result = scan_project_orphans_with_subjects(
        temp_project_root(),
        ProjectConfig::default(),
        vec![subject],
        vec![report],
        ScanOptions::default(),
    )
    .unwrap();

    assert_eq!(result.candidates[0].classification, OrphanClassification::Blocked);
}
```

Use the actual helper names from `src/core/orphan.rs`; keep helper functions private to
the test module when possible.

Verification:

```bash
cargo test mcp::tests::payload_contracts --lib
cargo test mcp::tests::orphan_handlers --lib
```

Expected:

- both test groups pass

## Task 6: Add End-To-End MCP Integration Test

Files:

- Modify: `tests/integration_test/mcp_session_reuse.rs`

Add one focused test:

- create temp project
- write `src/api/old.py`
- write `Dockerfile` referencing `src.api.old:app`
- register project through MCP
- call `scan_orphans` with a file subject for `src/api/old.py`
- assert first candidate is `blocked`
- assert `scanner_health` includes `entrypoint_scanner`

Use the existing `spawn_mcp_client`, `structured_payload`, and `CallToolRequestParams`
helpers already in that file.

Verification:

```bash
cargo test --test integration_test mcp_scan_orphans_blocks_entrypoint_referenced_file
```

Expected:

- the new integration test passes

## Task 7: Full Verification

Run:

```bash
cargo fmt --check
cargo test orphan:: --lib
cargo test mcp::tests::tool_surface --lib
cargo test mcp::tests::payload_contracts --lib
cargo test --test integration_test mcp_scan_orphans_blocks_entrypoint_referenced_file
```

Expected:

- all commands pass

If `cargo fmt --check` fails only due to formatting, run:

```bash
cargo fmt
cargo fmt --check
```

## Out Of Scope For This Plan

- storage tables for orphan scan runs
- `SCHEMA_VERSION` bump
- scanner command runner
- Python/FastAPI/pytest/TypeScript scanner implementations
- telemetry adapters
- CLI commands
- automatic deletion
