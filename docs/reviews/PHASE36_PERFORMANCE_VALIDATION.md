# Phase 36 Performance Validation

## Scope

This report records local performance validation after Phase 35.

Phase 36 introduces no runtime functionality, endpoint, table, migration,
policy behavior, replay behavior, review workflow, security behavior, or UI
behavior.

## Load Testing Report

A local HTTP read-load smoke ran the server with a file-backed SQLite database,
temporary workspace root, and temporary artifact root.

Command shape:

```text
cargo run server on 127.0.0.1:18081
8 concurrent workers
25 iterations per worker
GET /health/live
GET /health/ready
GET /metrics
```

Observed result:

```text
requests=600
workers=8
failures=0
```

This validates basic read-path availability under small concurrent local load.
It is not a production capacity benchmark.

## Concurrency Testing Report

Concurrency-sensitive coverage:

| Area | Evidence |
| :--- | :--- |
| Bounded work queue exists | `server::worker::tests::creates_bounded_run_queue` |
| SQLite work uses blocking boundary | `store::connection::with_connection` uses blocking tasks |
| File-backed pooled store works | `store::tests::pooled_connection_reuses_file_backed_store` |
| Request rate limiting rejects excess traffic | `server::security::limits::tests::rejects_requests_above_limit` |

Current concurrency model:

- HTTP submission uses a bounded queue.
- Worker execution is serialized by one background worker.
- Store access uses a file-backed pooled SQLite connection model in runtime.
- Each pooled operation opens a SQLite connection behind a semaphore.
- SQLite busy timeout is configured at 30 seconds.

## Queue Saturation Report

The run queue capacity is:

```text
RUN_QUEUE_CAPACITY = 64
```

Submission behavior under queue pressure:

- `POST /runs` creates a queued run record first.
- enqueue uses `try_send`.
- if enqueue fails, the run is marked `FAILED_INTERNAL`.
- the caller receives an unavailable response.

This prevents silent work loss, but it does not yet expose queue depth metrics.

Recommended future operator metric:

```text
acceptability_run_queue_depth
acceptability_run_queue_capacity
acceptability_run_queue_rejections_total
```

## Storage Performance Report

Storage-sensitive coverage:

| Area | Evidence |
| :--- | :--- |
| File-backed pooled SQLite store | `store::tests::pooled_connection_reuses_file_backed_store` |
| Production query indexes | `store::tests::creates_production_query_indexes` |
| Readiness path checks store reachability | `GET /health/ready` |
| Evidence replay uses read-only reconstruction | Phase 28 replay evidence |
| Backup and DR preserve replay output | Phases 33 and 34 evidence |

Current storage model:

- SQLite remains the authoritative evidence store.
- Runtime store access is file-backed.
- Pooled operations are bounded by a semaphore.
- Query indexes exist for production read paths.
- Artifact bytes remain in filesystem storage addressed by SQLite descriptors.

## Validation Commands

```text
cargo test creates_bounded_run_queue
cargo test pooled_connection_reuses_file_backed_store
cargo test creates_production_query_indexes
cargo test rejects_requests_above_limit
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
git diff --check
```

Local HTTP smoke:

```text
requests=600
workers=8
failures=0
```

## Notes / Deviations

- This phase validates local behavior, not production maximum throughput.
- No long-running soak test was executed.
- No multi-pod Kubernetes load test was executed.
- Queue depth is not yet exported as a metric.
- Gate execution remains intentionally serialized by the current worker model.

## Conclusion

Phase 36 establishes baseline local performance evidence for read availability,
queue behavior, request limiting, pooled SQLite access, and indexed storage
queries.

At the time of Phase 36, the project remained blocked from production release
by D25-001 candidate acquisition, not by a newly discovered Phase 36 performance
defect. The subsequent candidate acquisition implementation track closes
D25-001 for commit-SHA admission.
