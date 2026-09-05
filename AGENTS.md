# AGENTS.md

Universal engineering rules for AI coding agents working in this repository.
Scope: the entire repository. A nested `AGENTS.md` overrides these rules for its subtree; direct requests from the user override everything.

## Project

- **Stack:** Tauri v2 (Rust backend, `tauri` + `tray-icon`), React 19 + TypeScript + Vite 8 + Tailwind CSS v4 frontend. Windows-only native crates: `windows` 0.62 (Raw Input, message-only window) and `vigem-client` (ViGEmBus virtual Xbox 360 controllers); serde for config/profiles.
- **Source layout:** `src/` = React UI (Dashboard / Mapping editor / Profiles, shared types + typed IPC in `types.ts`/`api.ts`). `src-tauri/src/` = Rust: `lib.rs` (app wiring, tray), `core/` (serde model + canonical key names), `input/` (`devices.rs` enumeration, `capture.rs` Raw Input thread — Windows), `mapping/` (binding math), `controller/` (`vigem.rs` — Windows), `engine.rs` (single background engine thread), `commands.rs` (IPC), `state.rs`. `.github/workflows/` = CI + auto-release on version bumps.
- **Toolchain:** Bun (package manager), Rust stable with the MSVC toolchain on Windows. The app runs only on Windows 10/11 (Raw Input + ViGEmBus driver required); frontend/Rust checks also run from Linux/macOS.
- **Key docs:** `README.md` (usage, architecture, build & release flow), `.github/workflows/ci.yml` + `release.yml` (CI and bump-triggered releases), `AGENTS.md` (this file).

## Commands

- **Install:** `bun install` (root), then `cd src-tauri && cargo fetch` if needed
- **Build:** frontend `bun run build` · installers `bun run tauri build` (Windows only)
- **Dev server:** `bun run tauri dev` (Windows; runs Vite + desktop app) · `bun run dev` (Vite only)
- **Tests (full suite):** `cd src-tauri && cargo test`
- **Tests (single):** `cd src-tauri && cargo test <test-name>` (e.g. `cargo test mapping::`)
- **Lint (fix mode):** `bun run lint` · autofix with `bunx eslint . --fix`
- **Typecheck:** `bunx tsc -b`
- **Windows-only code check:** `cd src-tauri && cargo check --target x86_64-pc-windows-msvc`

If a command you need is missing, derive it from the repo's config files, then record it here.

## Workflow

1. **Understand** — read the change site, its tests, and its callers before editing. Confirm assumptions against the implementation; never infer behavior from names alone.
2. **Plan** — identify the files to change and the risk points. For architectural, breaking, or destructive changes, present the plan and wait for approval. Otherwise proceed directly.
3. **Implement** — deliver the smallest complete change that fulfills the request. Follow the conventions of each file you edit; reuse existing utilities before adding new ones. Leave no placeholders, stubs, or dead code.
4. **Verify** — run the relevant commands above until green. Fix what your change broke. Report pre-existing failures separately; leave them unasked.

## Conventions

- Write comments that explain intent, non-obvious behavior, and tradeoffs — the code itself explains the rest:

```text
// Retry twice: upstream rate-limits bursts (issue #1423)      <- good
// increment counter                                           <- noise
```

- Describe what changed in commit messages, never in comments.

## Commits

Use conventional commits: `type(scope): description`

```text
feat(api): add export endpoint
chore(deps): bump minor versions
```

For bug fixes, state the defect being repaired — name the method or component and describe the wrong behavior it had (use the "fixed a bug where ..." method):

```text
fix(auth): fixed a bug where refreshToken method reused expired tokens
```

One logical change per commit. Split mixed concerns into separate commits.

Strict rule: never append AI credits or generated-by footers to commits — no `Co-Authored-By: ...`, no `Generated with ...`, no tool signature lines of any kind. Commit messages are written by you alone.

## Dependencies

Exhaust existing packages and platform built-ins first. Before adding a dependency, confirm need, maintenance health, license compatibility, and security posture. Production additions require approval.

## Boundaries

**Always**
- on every commit and push request ask for version bump and CHANGELOG.md adding and then commit and push
- Run available checks (tests, lint, typecheck) before declaring work done.
- Report verification honestly: what was run, the result, and anything that could not be run and why.
- Add or update tests covering behavior you changed.

**Ask first**

- Schema, migration, or persisted-data-format changes — confirm a rollback path exists.
- New dependencies; edits to CI/CD, build, auth, or security configuration.
- Breaking changes to public APIs or exported signatures.
- Deleting seemingly unused code, files, or failing tests.

**Never**

- Commit secrets, credentials, tokens, or private keys — keep them in environment variables or git-ignored config.
- Force-push shared branches, rewrite published history, or bypass hooks and checks to force a pass.
- Modify `AGENTS.md`, licenses, or changelogs unless explicitly asked.
- Silence errors: empty catch blocks, swallowed exceptions, suppressed type errors, deleted assertions.
- Claim verification that was not actually executed.

## Data changes

Before changing schemas or storage formats: check backward compatibility, write the migration, verify the rollback works. Destructive data operations require explicit approval.

## Versioning

When the project tracks releases, apply semver: **major** = breaking, **minor** = backwards-compatible feature, **patch** = fix. Choose the highest applicable level, keep related artifacts in sync, and skip bumps for documentation-only changes.

---

**Definition of done** — confirm before finishing any change: focused diff ✓ · checks pass ✓ · honest report delivered ✓ · nothing touched beyond the request ✓.

**On every commit or push request** - on every commit and push request ask for version bump and CHANGELOG.md adding, ask for creating a tag and release, ask for release notes written in user-friendly language (and for bug fixes use the "fixed a bug where ..." method), and then commit and push
