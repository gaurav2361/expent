# Gemini CLI Core Mandates

## Contextual Precedence

The instructions in this file are foundational and take absolute precedence over general workflows for the Gemini CLI.

## Workflow Priorities

1. **Security First**: Protect `.env`, `.git`, and system configs. Never commit secrets.
2. **Context Efficiency**: Combine tool calls. Use `wait_for_previous: true` only when necessary.
3. **Engineering Integrity**: Adhere to existing patterns. Use `SmartMerge` for DB operations.
4. **Validation**: Run `cargo check -p server`, `cd apps/dashboard && pnpm tsc`, or `uv run pytest` after changes.

## Test-Driven Development (TDD)

- **TDD Cycle**: Mandatory Red-Green-Refactor cycle for all new development.
- **Function Coverage**: Create comprehensive tests for EVERY new function.
- **Legacy Integrity**: Write tests for existing functions to formalize intended behavior.
- **Continuous Validation**: Execute relevant tests every few minutes during development.
- **Verification First**: Code is NOT complete until verified by exhaustive automated tests.

## Performance Optimization

- **Parallelism**: Run independent search/read/edit tasks in the same turn.
- **Surgical Edits**: Use `replace` with enough context to avoid ambiguity.
- **Discovery**: Prefer `grep_search` and `glob` over manual traversal.

## Constraints & Standards

- **No Filler**: Keep responses concise and technical.
- **Dependency Audit**: Verify established usage in `package.json`/`Cargo.toml` before adding libs.
- **Atomic Commits**: Fulfill the request thoroughly, including related tests and verification logic.
- **Commit Messages**: CLEAR, CONCISE, and focused on "WHY". Propose draft messages to the user.
