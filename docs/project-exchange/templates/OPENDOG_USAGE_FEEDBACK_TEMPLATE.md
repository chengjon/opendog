# OpenDog Usage Feedback Template

> Purpose:
> Use this template to record how OpenDog behaves in a target project.
> Keep the project-local report short if needed, but preserve the final report under `docs/project-exchange/reports/<project>/` in the OpenDog repository.

## Project Metadata

- Project name:
- Project id:
- Project path:
- Primary language / framework:
- Approximate file count:
- OpenDog entrypoints used: MCP / CLI / daemon
- OPENDOG_HOME:
- Report date:
- Reporter:

## Communication Routing

- Source project: `reports/<project>/`
- OpenDog response owner:
- Response file or section:
- Shared issue ids linked from this report:
  - `ODX-YYYYMMDD-<slug>`

## Daily Usage Record

```md
### YYYY-MM-DD - Short Title

- Goal:
- Entrypoint: MCP / CLI
- Project state:
  - Snapshot fresh: yes / no / unknown
  - Monitor running: yes / no / unknown
  - Verification evidence: missing / stale / fresh
- Commands or MCP tool calls:
- Expected behavior:
- Actual behavior:
- What helped:
- Friction or confusion:
- Workaround:
- Impact on real work:
- Follow-up:
```

## Tuning Evidence Record

```md
### YYYY-MM-DD - Tuning Item

- Type: bug / design gap / UX friction / performance / classification noise / documentation gap
- Severity: low / medium / high
- Stable reproduction: yes / no / unknown
- Affected entrypoint: MCP / CLI / daemon / mixed
- Project facts:
  - Snapshot file count:
  - Accessed file count:
  - Unused file count:
  - Monitor running:
- Environment:
  - Host:
  - MCP host / terminal:
  - OPENDOG_HOME:
- Exact steps:
  1.
  2.
  3.
- Expected behavior:
- Actual behavior:
- Impact:
- Evidence:
  - Command or tool call:
  - Output summary:
  - Artifact path:
- Hypothesis:
- Suggested OpenDog improvement:
- Shared issue id:
- OpenDog response:
  - Status: new / accepted / fixed / deferred / rejected
  - Response summary:
  - Fix or mitigation link:
  - Applies to: this project only / all projects / listed projects
```

## Monthly Summary

```md
## YYYY-MM Summary

- Most useful OpenDog behavior:
- Main friction:
- Repeated false positives or noise:
- CLI vs MCP division of work:
- Evidence volume:
  - Total tuning cases:
  - Stable reproduction cases:
  - Cases that disappeared or could not be reproduced:
- Highest-value improvement requests for next cycle:
- Evidence links:
```

## Safety Boundaries

- Do not paste secrets, tokens, keys, account credentials, or private customer data.
- Do not treat `unused` as proof that a file is dead or safe to delete.
- Confirm cleanup/refactor decisions with repository context, verification evidence, and direct shell inspection.
