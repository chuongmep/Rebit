# Rebit

Rebit is a next-generation BIM/CAD platform being built in Rust.

This repository is set up as a contributor-ready monorepo so multiple engineering teams can work in parallel with clear ownership and quality gates.

## Current Status

- Rust workspace scaffold is initialized.
- Core architecture crates are created.
- Desktop and cloud app entrypoints are created.
- CI, CODEOWNERS, and contribution standards are in place.

## Core Project Scaffold

Rust monorepo structure is now initialized for parallel team contribution:

- Workspace manifest: [Cargo.toml](Cargo.toml)
- Contributor guide: [CONTRIBUTING.md](CONTRIBUTING.md)
- Ownership rules: [CODEOWNERS](CODEOWNERS)
- Architecture and role tasks: [hiring-system/08_rust_architecture_and_role_tasks.md](hiring-system/08_rust_architecture_and_role_tasks.md)
- CI pipeline: [.github/workflows/ci.yml](.github/workflows/ci.yml)
- Bootstrap script: [scripts/bootstrap.sh](scripts/bootstrap.sh)

## Repository Layout

- [apps/desktop](apps/desktop): desktop application entrypoint
- [apps/cloud_api](apps/cloud_api): cloud API entrypoint
- [crates](crates): core domain crates (kernel, BIM, rendering, interop, cloud, SDK, quality)
- [docs/adr](docs/adr): architecture decision records
- [docs/rfc](docs/rfc): design proposals and technical contracts
- [docs/runbooks](docs/runbooks): operational runbooks
- [hiring-system](hiring-system): hiring and org execution package

## Quickstart for Engineers

1. Install Rust toolchain from [rust-toolchain.toml](rust-toolchain.toml).
2. Validate workspace:
   - `cargo check --workspace`
3. Run tests:
   - `cargo test --workspace`
4. Run lint and format checks:
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo fmt --all -- --check`
5. Or run all checks:
   - `./scripts/bootstrap.sh`

## Engineering Rules of Engagement

1. Follow [CONTRIBUTING.md](CONTRIBUTING.md) for PR and review standards.
2. CODEOWNERS approval is required for owned crate changes.
3. API or architecture changes require RFC/ADR updates.
4. Every change must include tests at the correct boundary.

## Architecture and Role Ownership

- Canonical architecture and role-task mapping: [hiring-system/08_rust_architecture_and_role_tasks.md](hiring-system/08_rust_architecture_and_role_tasks.md)
- Role-by-role job descriptions and scope of work: [hiring-system/roles/README.md](hiring-system/roles/README.md)
- Role priority matrix: [hiring-system/05_role_priority_matrix.md](hiring-system/05_role_priority_matrix.md)
- Weekly execution cadence: [hiring-system/02_operating_cadence.md](hiring-system/02_operating_cadence.md)

## Hiring and Organization Implementation

Execution documents for the 6-month hiring and organization plan are in:

- [hiring-system/IMPLEMENTATION.md](hiring-system/IMPLEMENTATION.md)
- [hiring-system/01_roadmap_6_months.md](hiring-system/01_roadmap_6_months.md)
- [hiring-system/02_operating_cadence.md](hiring-system/02_operating_cadence.md)
- [hiring-system/03_interview_system.md](hiring-system/03_interview_system.md)
- [hiring-system/04_recruiting_pipeline_model.md](hiring-system/04_recruiting_pipeline_model.md)
- [hiring-system/05_role_priority_matrix.md](hiring-system/05_role_priority_matrix.md)
- [hiring-system/06_risk_register.md](hiring-system/06_risk_register.md)
- [hiring-system/07_weekly_hiring_review_template.md](hiring-system/07_weekly_hiring_review_template.md)
- [hiring-system/08_rust_architecture_and_role_tasks.md](hiring-system/08_rust_architecture_and_role_tasks.md)
- [hiring-system/dashboards/hiring_dashboard_template.csv](hiring-system/dashboards/hiring_dashboard_template.csv)

Templates:

- [hiring-system/templates/job_description_template.md](hiring-system/templates/job_description_template.md)
- [hiring-system/templates/role_scorecard_template.md](hiring-system/templates/role_scorecard_template.md)
- [hiring-system/templates/interview_feedback_template.md](hiring-system/templates/interview_feedback_template.md)

## Suggested Rollout Order

1. Confirm role priorities in [hiring-system/05_role_priority_matrix.md](hiring-system/05_role_priority_matrix.md).
2. Launch cadence from [hiring-system/02_operating_cadence.md](hiring-system/02_operating_cadence.md).
3. Standardize interviews with [hiring-system/03_interview_system.md](hiring-system/03_interview_system.md).
4. Track weekly progress with [hiring-system/dashboards/hiring_dashboard_template.csv](hiring-system/dashboards/hiring_dashboard_template.csv).
5. Run weekly review using [hiring-system/07_weekly_hiring_review_template.md](hiring-system/07_weekly_hiring_review_template.md).
6. Execute architecture ownership and per-role tasks from [hiring-system/08_rust_architecture_and_role_tasks.md](hiring-system/08_rust_architecture_and_role_tasks.md).
