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
- Linear Project ID: d7e9b106-10c4-4b6d-9607-28a732861d19
- GitHub Repo: Day1-Doctor/d1-doctor-app
- Branch naming: `feature/D1D-{N}-short-description`
- Default branch: `main`

## Worker Notes
- Local env setup: `npm install` + `cargo build`
- Common gotchas: Tauri requires system deps (webkit2gtk on Linux)
- Files to never modify automatically: `tauri.conf.json` (app signing config)

<!-- mf-infra-context -->
## Infrastructure Context

### Cloud & Environments
**Providers:** GCP (Cloud Run, Artifact Registry, Memorystore Redis, Monitoring) · Supabase (database + auth + vault) — Region: us-central1

| Environment | Auto-Deploy | Approval | URL |
|-------------|-------------|----------|-----|
| dev | yes | no | http://localhost:1420 |
| staging | no | yes | TBD (planned) |
| prod | no | yes | https://day1.doctor |

### Services
| Service | Repo | Runtime | Health |
|---------|------|---------|--------|
| platform | d1-doctor-platform | cloud-run (FastAPI) | /health |
| gateway | d1-doctor-platform | cloud-run (LiteLLM) | /health |
| desktop-app | d1-doctor-app | tauri (Rust+Vue3) | — |

### Infrastructure-as-Code
- **IaC tool:** Terraform (>= 1.5, google provider ~> 5.0)
- **Infra repo:** d1-doctor-platform
- **State backend:** GCS (`d1-doctor-terraform-state`)
- **Modules:** N/A
- **Environments:** `infra/terraform/environments`

### Deployment Workflow
- **Platform + Gateway:** Docker build → Artifact Registry → Cloud Run (via `cd.yml`)
- **Dev:** auto-deploy on merge to main
- **Prod:** manual approval gate (`production` GitHub environment)
- **Image tags:** `sha-{8-char-commit-hash}`
- **Rollback:** `rollback.yml` with `workflow_dispatch` (env + service + optional tag)

### Database
- **Provider:** Supabase (Postgres)
- **Migrations:** `TBD` in `d1-doctor-platform`
- **Migration commands:** `supabase db push` / `supabase migration new`

### Secrets
- **Store:** GCP Secret Manager
- **Scanning:** Enabled (secrets-validation.yml on PRs)
- **Rotation:** Manual

### Infrastructure Decision Log
<!-- Append decisions here so future sessions have context -->
| Date | Decision | Rationale |
|------|----------|-----------|

### Known Gotchas
<!-- Things that would prevent a worker from getting stuck on infra tasks -->
None documented yet.
<!-- end mf-infra-context -->
