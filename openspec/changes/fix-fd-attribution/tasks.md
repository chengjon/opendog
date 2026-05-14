## 1. Spec and design

- [x] 1.1 Finalize the `fd-attribution` proposal, spec, and design artifacts.
- [x] 1.2 Validate the change structure with OpenSpec.

## 2. Scanner implementation

- [x] 2.1 Update `/proc/<pid>/fd` scanning to distinguish file-level and directory-level targets before sighting emission.
- [x] 2.2 Add scan-cycle deduplication so the same fd cannot inflate access counts.
- [x] 2.3 Keep the scanner output compatible with the existing monitor/stat storage path.

## 3. Tests

- [x] 3.1 Add unit coverage for directory fd exclusion.
- [x] 3.2 Add unit coverage for duplicate fd observations within one scan cycle.
- [x] 3.3 Add a regression test that exercises the attribution path with representative file targets.

## 4. Large-repo verification

- [x] 4.1 Run the Rust test suite after the attribution change.
- [x] 4.2 Run a `mystocks`-scale validation and confirm `.py` / `.vue` source files retain distinct access counts.
- [x] 4.3 Record the verification outcome and any residual attribution noise.

## 5. Governance closure

- [x] 5.1 Mark `fix-fd-attribution` as the accepted scanner attribution baseline.
- [x] 5.2 Split the unrelated `agent-guidance` UTF-8 boundary panic into an independent governed task.
- [x] 5.3 Require future scanner attribution semantic changes to use OpenSpec governance before acceptance.
