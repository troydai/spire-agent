# Repository Guidelines

## Project Structure & Module Organization
- Workspace root is a Rust workspace (`Cargo.toml`) with two crates.
- `spire-agent/` holds the primary agent implementation (`src/`) and build script (`build.rs`).
- `spire-agent-mock/` is a mock SPIRE Workload API server for local testing, with scripts in `spire-agent-mock/scripts/`.
- Protobuf definitions live at `spire-agent/proto/workload.proto` and `spire-agent-mock/proto/workload.proto`.

## Build, Test, and Development Commands
- `cargo build` builds the full workspace.
- `cargo build -p spire-agent` builds only the agent crate.
- `cargo build -p spire-agent-mock` builds only the mock server.
- `cargo test` runs all tests across both crates.
- `spire-agent-mock/scripts/start.sh` builds and launches the mock server (defaults to `/tmp/agent.sock`).
- `spire-agent-mock/scripts/test.sh` uses `grpcurl` to exercise mock APIs against a running socket.

## Coding Style & Naming Conventions
- Rust style follows `rustfmt` defaults (4-space indentation, trailing commas, etc.).
- Use `snake_case` for functions/modules, `CamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants.
- Keep module boundaries tight and prefer explicit `pub` surfaces only where needed.

## Testing Guidelines
- Tests are standard Rust unit tests colocated with modules (e.g., `spire-agent/src/fetch_x509.rs`).
- Prefer unit tests for parsing/encoding and helper utilities; add integration tests only if cross-crate behavior is required.
- Run `cargo test -p spire-agent` or `cargo test -p spire-agent-mock` to focus on one crate.

## Commit & Pull Request Guidelines
- Commit subjects use imperative, title-style phrasing and often include an issue/PR reference like `(#14)`.
- Keep commits scoped to a single change and note behavior or API shifts in the message.
- PRs should include: a short summary, testing performed (commands), and linked issues when applicable.
- If changing the mock server or proto files, call out any client-visible behavior changes.

## Security & Configuration Tips
- UDS socket paths default to `/tmp/agent.sock`. Override via env vars in the mock: `SPIRE_MOCK_SOCKET_PATH`, `SPIRE_MOCK_X509_INTERNAL_SECONDS`.
- Avoid committing generated certs or sockets; keep runtime artifacts out of git.

## Agent workflow
- Work in tight iterative loops.
- Commit frequently, often after completing small amounts of work.
- Utilize the pre-commit hook to enforce unit tests and linting.
- Catch up remote branch often, use the Pull Request as the main vehicle to communicate the changes.
- Test the work after each change.
