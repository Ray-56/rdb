# rdb

A pure Rust database implementation with no external database kernels.

## Project Principles

This project strictly follows Domain-Driven Design (DDD) architecture and Rust best practices. See the [Project Constitution](.specify/memory/constitution.md) for complete architectural principles and development standards.

### Key Design Principles

1. **Pure Rust**: All code is pure Rust with zero external database kernels or FFI dependencies
2. **DDD Architecture**: Clean separation of Domain, Application, Infrastructure, and Interface layers
3. **Memory Safety**: Strict ownership and lifetime correctness, unsafe only in low-level storage operations
4. **MVCC-Ready**: Designed from day one to support Multi-Version Concurrency Control
5. **Property Testing**: All domain entities have proptest property-based tests
6. **Cluster-Ready**: Storage format designed to support future distributed deployments

## Development

This project uses the [speckit](https://github.com/your-org/speckit) workflow for feature development. All features must comply with the project constitution.
