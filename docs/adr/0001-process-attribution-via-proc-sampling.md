# ADR 0001: Process Attribution via `/proc` Sampling

Date: 2026-05-27

## Status

Accepted

## Context

OPENDOG needs to identify which files AI tools are using. Linux inotify reports filesystem changes but does not include the user or process that triggered an event. Designing around PID data from inotify would make the observation model impossible to implement correctly.

## Decision

Use periodic `/proc/<pid>/fd` sampling for process/file attribution and use inotify/`notify` as secondary file-change evidence.

## Consequences

- File usage attribution is approximate and sampling-based.
- Short-lived file opens may be missed.
- Sustained AI process attention can be ranked with useful confidence.
- Inotify evidence should not be described as process-attributed evidence.
- Guidance must stay honest about evidence freshness, sampling windows, and attribution limits.
