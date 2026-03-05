# d1-doctor-app — Worker Context

## Stack
- **Language:** Rust + TypeScript
- **Framework:** Tauri 2.x + Vue 3
- **Test command:** `npm test`
- **Lint command:** `npm run lint`
- **Build command:** `cargo build` (Rust) / `npm run build` (frontend)

## Project Structure
```
crates/         Rust crates (desktop, daemon, sdk, cli, common)
docs/           Design docs and plans
examples/       Example configs / usage
proto/          Protobuf definitions (shared)
scripts/        Build & dev scripts
benchmark/      Performance benchmarks
```

## Conventions
- Tauri commands live in `crates/desktop/src/`
- Daemon (local agent runtime) in `crates/daemon/`
- Frontend Vue SFCs follow `<script setup lang="ts">` style
- WebSocket protocol v1: `{ v:1, id, ts, type, payload }`

## Before Creating a PR
1. `npm test` — all must pass
2. `npm run lint` — zero new errors
3. Run mf-pr-reviewer agent — must output REVIEW_PASSED
4. Verify: `git diff --stat main` — no unrelated files

## Linear & GitHub
- Linear Project ID: TBD
- GitHub Repo: Day1-Doctor/d1-doctor-app
- Branch naming: `feature/{TEAM-ID}-short-description`
- Default branch: `main`

## Worker Notes
- Local env setup: `npm install` + `cargo build`
- Common gotchas: Tauri requires system deps (webkit2gtk on Linux)
- Files to never modify automatically: `tauri.conf.json` (app signing config)
