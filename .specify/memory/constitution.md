<!--
Sync Impact Report:
- Version change: [NEW] → 1.0.0
- Initial constitution creation
- Principles defined: 7 core architectural and engineering principles
- Templates requiring updates:
  ✅ plan-template.md - reviewed, constitution check section compatible
  ✅ spec-template.md - reviewed, requirements section compatible
  ✅ tasks-template.md - reviewed, task structure compatible
- Follow-up TODOs: None
- Notes: First ratification of rdb constitution. All 7 principles are NON-NEGOTIABLE.
-->

# rdb Constitution

## Core Principles

### I. Domain-Driven Design Architecture (NON-NEGOTIABLE)

All code MUST follow Domain-Driven Design (DDD) layered architecture principles:

- Clear separation between Domain, Application, Infrastructure, and Interface layers
- Domain layer contains pure business logic with no external dependencies
- Application layer orchestrates use cases and domain operations
- Infrastructure layer handles persistence, I/O, and external integrations
- Interface layer provides CLI, API, or other entry points

**Rationale**: DDD ensures maintainability, testability, and enables the system to evolve with changing requirements. Layered architecture prevents tight coupling and makes the codebase comprehensible at scale.

### II. Pure Rust Implementation (NON-NEGOTIABLE)

All code MUST be pure Rust with zero external database kernels or runtime dependencies:

- No embedded C/C++ database engines (no SQLite, RocksDB, etc.)
- No FFI calls to external storage libraries
- All storage, indexing, and query processing implemented in Rust
- Only standard Rust dependencies (std, core, alloc) plus carefully vetted crates

**Rationale**: Pure Rust implementation ensures memory safety, predictable performance, and complete control over the entire system behavior. This principle is fundamental to the project's identity and goals.

### III. Memory Safety & Correctness (NON-NEGOTIABLE)

Ownership, lifetimes, and concurrency primitives MUST be strictly correct:

- All Send/Sync bounds must be explicitly justified and correct
- unsafe code ONLY permitted in Pager and B+Tree page pointer operations
- Every unsafe block MUST have detailed safety comments explaining invariants
- All unsafe code MUST be reviewed and justified during code review
- Use static analysis tools (Miri, sanitizers) to verify unsafe code correctness

**Rationale**: As a database system, correctness is paramount. Memory safety bugs lead to data corruption and undefined behavior. Restricting unsafe to low-level storage operations provides a clear safety boundary.

### IV. MVCC Interface (NON-NEGOTIABLE)

System MUST reserve interfaces for Multi-Version Concurrency Control:

- Storage layer must support versioned records from day one
- API surface must accommodate snapshot isolation primitives
- Data structures should be designed with temporal queries in mind
- Page format and record layout must include version metadata placeholders

**Rationale**: MVCC is essential for production database systems. Retrofitting MVCC into an existing architecture is extremely difficult. Reserving the interface now prevents costly rewrites later.

### V. Clean API Boundaries (NON-NEGOTIABLE)

All internal modules MUST use pub(crate) visibility; only expose clean public APIs:

- Public API surface must be minimal and well-documented
- Internal implementation details hidden behind module boundaries
- Use trait-based abstractions for extensibility points
- No leaking of internal types (Page, Node, etc.) into public interfaces

**Rationale**: Clear API boundaries enable internal refactoring without breaking users. This also enforces modularity and prevents accidental dependencies on implementation details.

### VI. Property-Based Testing (NON-NEGOTIABLE)

Every domain entity MUST have proptest property tests:

- Use proptest to generate random test cases for all domain types
- Properties must verify invariants (e.g., B+Tree balance, ordering)
- Test edge cases automatically (empty, single element, overflow, etc.)
- Property tests required before merging any domain entity

**Rationale**: Property-based testing catches edge cases that example-based tests miss. For a database, correctness across all possible inputs is critical. Proptest provides mathematical confidence in invariants.

### VII. Cluster-Ready Storage Format (NON-NEGOTIABLE)

Storage format design MUST be 100% compatible with future distributed/cluster versions:

- Page format must support replication metadata (LSN, term, replica ID)
- File headers must include version and feature flags for forward compatibility
- All on-disk structures must be deterministic and byte-comparable
- Design with consensus protocols in mind (Raft log entries, snapshot transfer)

**Rationale**: Storage format changes require complex migrations. Designing for clustering upfront avoids breaking changes that force users to rebuild databases. This principle protects future extensibility.

## Development Standards

### Code Quality

- All code must pass clippy with no warnings (`cargo clippy -- -D warnings`)
- Use rustfmt with project configuration (enforce formatting in CI)
- Documentation required for all public APIs (deny missing_docs where applicable)
- Integration tests for all user-facing features

### Performance Requirements

- Benchmark all critical paths (page I/O, B+Tree operations, query execution)
- Use criterion for performance regression testing
- Profile regularly with flamegraph and perf tools
- Document performance characteristics in module docs

### Error Handling

- Use Result types for all fallible operations
- Define clear error hierarchies (use thiserror or similar)
- Errors must be actionable and user-friendly at API boundaries
- Never use unwrap() or expect() in production code paths

## Governance

This constitution supersedes all other development practices and guidelines. All code changes, design decisions, and architectural proposals MUST comply with these principles.

### Amendment Process

1. Proposed amendment documented with rationale and impact analysis
2. Review by maintainers with explicit approval required
3. Migration plan documented if changes affect existing code
4. Version bump (MAJOR for breaking changes to principles)

### Compliance Review

- All pull requests MUST verify compliance with constitution principles
- Reviewers must explicitly check for unsafe code justification
- Property tests must be present for new domain entities
- Any complexity or principle violation must be explicitly justified

### Enforcement

- CI pipeline must enforce formatting, linting, and test requirements
- Breaking constitution principles requires documented justification
- Unjustified violations will not be merged
- Constitution principles may be cited in code review feedback

**Version**: 1.0.0 | **Ratified**: 2025-12-10 | **Last Amended**: 2025-12-10
