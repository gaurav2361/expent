# Gemini CLI Core Mandates

## Contextual Precedence

The instructions in this file are foundational and take absolute precedence over general workflows for the Gemini CLI.

## Workflow Priorities

1. **Security First**: Protect `.env`, `.git`, and system configs. Never commit secrets.
2. **Context Efficiency**: Combine tool calls. Use `wait_for_previous: true` only when necessary.
3. **Engineering Integrity**: Adhere to existing patterns. Use `SmartMerge` for DB operations.
4. **Validation**: Run `cargo check -p server`, `cd apps/dashboard && pnpm tsc`, `vitest run`, or `uv run pytest` after changes.

## Testing Strategy & TDD

- **TDD Cycle**: Mandatory Red-Green-Refactor cycle for all new development.
- **API-Heavy Philosophy**: Prioritize testing backend logic over frontend UI.
  - **Rust Backend**: Heavy emphasis on core logic unit testing (`expent_core`) and API endpoint integration testing using `rstest`. Parameterize edge cases for all financial math.
  - **TypeScript Frontend**: Use `vitest` strictly for headless utility functions, state, and complex hooks in `apps/dashboard`. Do NOT write UI component tests or browser-based E2E tests unless explicitly requested.
- **Function Coverage**: Create comprehensive tests for EVERY new backend function before it is considered complete.

## Performance Optimization

- **Parallelism**: Run independent search/read/edit tasks in the same turn.
- **Surgical Edits**: Use `replace` with enough context to avoid ambiguity.
- **Discovery**: Prefer `grep_search` and `glob` over manual traversal.

## Constraints & Standards

- **No Filler**: Keep responses concise and technical.
- **Dependency Audit**: Verify established usage in `package.json`/`Cargo.toml` before adding libs.
- **Atomic Commits**: Fulfill the request thoroughly, including related tests and verification logic.
- **Commit Messages**: CLEAR, CONCISE, and focused on "WHY". Propose draft messages to the user.
