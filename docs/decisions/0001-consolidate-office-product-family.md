# ADR-0001: Consolidate the GoreeCloud Office product family

## Status

Accepted — September 26, 2026

## Context

GoreeCloud Office was initially represented by a shared `goreecloud-office` repository plus separate `writer`, `spreadsheet`, `presentations`, `forms`, `database`, `office-server`, `office-formats`, and `office-templates` repositories. The satellite repositories contained only placeholder or near-placeholder content while substantive shared Office Engine and format work was already occurring in the umbrella repository.

The active GoreeCloud Repository Architecture, Boundaries, and Organization standard defines Office as a product-family monorepo candidate and directs tightly coupled Office applications, services, formats, templates, collaboration infrastructure, common models, and tooling toward a shared repository when separate repositories do not represent independent lifecycles.

## Decision

Use this repository as the canonical GoreeCloud Office product-family monorepo.

Component boundaries are represented internally:

- `apps/writer/`
- `apps/spreadsheet/`
- `apps/presentations/`
- `apps/forms/`
- `apps/database/`
- `services/office-server/`
- `packages/formats/`
- `packages/templates/`

Existing shared Office Engine crates and infrastructure remain at their current repository locations until a later source-layout refactor is independently justified.

Applications and services may retain distinct build targets, release artifacts, tests, APIs, and runtime boundaries while sharing one source repository.

## Alternatives Considered

### Keep all repositories separate

Rejected because the satellite repositories did not have meaningful independent development or release lifecycles and separation created administrative fragmentation.

### Create a new empty `office` repository and migrate everything immediately

Not selected because the existing Office repository already contained the substantive Office Engine source. Consolidation therefore preserved that repository and its source, and the provider repository was subsequently renamed in place to `GoreeCloud/office`.

## Consequences

- New Office-family work should land in this repository unless a component later develops a genuine independent lifecycle.
- Cross-component changes can be reviewed atomically.
- Placeholder predecessor repositories become migration predecessors rather than active development locations.
- The canonical repository was renamed in place to `GoreeCloud/office`, and the eight former standalone Office component repositories were archived as migration predecessors on September 26, 2026.
