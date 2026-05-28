# fd-attribution Specification

## Purpose
Define the scanner contract for attributing `/proc/<pid>/fd` observations to project files. These requirements prevent directory descriptors from fanning out into false per-file activity while preserving regular-file sightings, scan-cycle deduplication, and large-repository regression evidence.
## Requirements
### Requirement: File-level fd attribution only
The system SHALL count usage only from file-level fd targets that resolve to files inside the project snapshot set. Directory-level fd targets MUST NOT create per-file access_count updates for the files under that directory.

#### Scenario: Directory fd does not fan out
- **WHEN** the scanner encounters a directory fd for a project path that contains multiple snapshot files
- **THEN** the scan MUST NOT increment access_count for each file beneath that directory

#### Scenario: Regular file fd is counted
- **WHEN** the scanner encounters a regular file fd that resolves to a snapshot file in the project root
- **THEN** the scan SHALL record a sighting for that file

### Requirement: Scan-cycle fd deduplication
The system SHALL ensure that the same fd contributes at most one sighting per scan cycle for a given process, so repeated observations of the same fd do not inflate per-file usage counts.

#### Scenario: Same fd observed more than once
- **WHEN** the scanner sees the same process fd multiple times during one scan cycle
- **THEN** the system SHALL record at most one sighting for that fd in that cycle

#### Scenario: Duplicate path resolution does not double count
- **WHEN** two resolution paths converge on the same fd during one scan cycle
- **THEN** the system SHALL not produce duplicate access_count updates for that fd

### Requirement: Large-repository regression preserves distinct source-file counts
The system SHALL support regression validation on a large repository such that independently opened source files retain distinct access counts and are not collapsed into a shared directory-driven count.

#### Scenario: Mystocks regression after fix
- **WHEN** the scanner is exercised against the `mystocks` repository after the fix
- **THEN** independently opened `.py` and `.vue` source files SHALL be able to receive different access_count values rather than sharing one directory-fan-out count
- **AND** the regression evidence SHALL show that the prior identical-count pattern no longer holds for the affected source files
