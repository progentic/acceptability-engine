# License Governance

## Purpose

This document records the v1.0 dependency license policy for the Rust authority plane.

## Validation Command

Run from the Rust package root:

```text
cargo deny check
```

The check must validate advisories, bans, sources, and licenses.

## Approved Licenses

The approved license list is derived from the current locked dependency graph:

```text
Apache-2.0
BSD-2-Clause
BSD-3-Clause
MIT
Unicode-3.0
Unlicense
```

Dependencies using these licenses may be admitted when `cargo deny check` passes.

## Licenses Requiring Review

Any license absent from the approved list requires review before the dependency may be used.

Review requires:

```text
dependency name
dependency version
license expression
reason for use
approver
approval date
```

Approved exceptions must be recorded in this document and encoded in `core/deny.toml`.

## Prohibited Licenses

Licenses that fail `cargo deny check` without a documented exception are prohibited.

Unknown registries and unknown Git sources are prohibited.

## Current Exceptions

No license exceptions are approved.

## D37-001 Closure Evidence

D37-001 is closed by:

```text
core/deny.toml exists
cargo deny check passes
the local crate declares its license
the approved license list is documented
the exception process is documented
Phase 37 and PHASEMAP record the closure
CHANGELOG records the governance change
```
