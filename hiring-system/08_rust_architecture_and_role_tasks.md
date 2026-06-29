# Rust Architecture and Role Task Map

## Goal

Design a Rust-first architecture that allows all engineering teams to contribute safely in parallel, with clear ownership and explicit tasks per role.

## Architecture principles

1. Rust-first across desktop core, geometry engine, cloud services, and SDK runtime.
2. Domain isolation via crate boundaries and stable interfaces.
3. Parallel contribution through contract-driven development.
4. Deterministic behavior for geometry, constraints, and parametrics.
5. Performance and correctness gates in CI from day one.

## Monorepo structure

```text
rebit/
  Cargo.toml                  # workspace root
  crates/
    core_math/                # numeric primitives, tolerances
    geometry_kernel/          # B-Rep, booleans, topology
    constraint_solver/        # geometric and dimensional solver
    bim_model/                # BIM entity graph and schema
    parametric_engine/        # dependency graph, formula evaluation
    scene_graph/              # render graph structures
    rendering_engine/         # renderer front-end and draw orchestration
    gpu_backend/              # Vulkan/Metal adapters and shader runtime
    ui_framework/             # command bus, tool state, panel framework
    desktop_shell/            # native host integration, files, prefs
    file_ifc/                 # IFC parser/writer and mapping
    file_dwg_bridge/          # DWG interoperability boundary
    import_export_pipeline/   # translation jobs and diagnostics
    auth_identity/            # OIDC, tenancy, RBAC
    cloud_project_service/    # projects, model versions, comments
    collaboration_service/    # async collaboration workflows
    plugin_sdk/               # plugin API and runtime contracts
    ai_workflows/             # AI-assisted command and validation flows
    perf_harness/             # benchmark scenarios and perf assertions
    test_oracles/             # golden geometry outputs and invariants
    telemetry/                # logs, traces, metrics
  apps/
    desktop/                  # desktop binary
    cloud_api/                # backend API binary
  tools/
    codegen/
    schema_lint/
    release_tools/
  docs/
    adr/
    rfc/
    runbooks/
```

## Architecture layers and ownership

| Layer | Key crates | Primary owner | Secondary owners |
|---|---|---|---|
| Numeric and geometry foundation | core_math, geometry_kernel | Geometry Kernel team | Constraint, Performance |
| Parametric reasoning | constraint_solver, parametric_engine | Constraint + Parametric teams | BIM Model |
| BIM domain model | bim_model | BIM Data Model team | Interop, SDK |
| Rendering and interaction | scene_graph, rendering_engine, gpu_backend | Rendering + GPU teams | UI Framework |
| Desktop experience | ui_framework, desktop_shell | UI + Desktop teams | SDK |
| Interoperability | file_ifc, file_dwg_bridge, import_export_pipeline | Interop teams | BIM Model |
| Cloud and identity | auth_identity, cloud_project_service, collaboration_service | Cloud/Auth/Collab teams | Security |
| Ecosystem and AI | plugin_sdk, ai_workflows | SDK + AI teams | Desktop, BIM |
| Quality and platform | perf_harness, test_oracles, telemetry | QA + Platform teams | All teams |

## Contribution model for all engineering

### Branch and review model

1. Trunk-based development with short-lived feature branches.
2. Mandatory CODEOWNERS review for crate owners.
3. Cross-domain reviewer required for any public API change.
4. ADR required for architecture-affecting changes.

### Contract-driven development

1. Each crate exposes an interface contract in docs/rfc.
2. Upstream and downstream compatibility tests run in CI.
3. Breaking changes require migration plan and version bump.

### Quality gates in CI

1. Format and lint: rustfmt, clippy, security lint.
2. Unit and property tests for each crate.
3. Integration tests at layer boundaries.
4. Geometry golden tests and invariant checks.
5. Performance budgets on startup, file load, and interaction latency.

### Cross-team contribution lanes

1. Any engineer can contribute to any crate after passing domain checklist.
2. Non-owner changes need one owner approval and one platform approval.
3. Shared monthly hardening sprint for cross-team refactors.

## Technical decision flow

1. RFC for major design and API changes.
2. ADR for accepted decisions and rejected alternatives.
3. Principal Council weekly review for high-risk decisions.
4. Decision SLA: 48 hours for P0 topics, 5 days for P1 topics.

## Role-to-architecture task matrix

Tasks are grouped into three windows:

- Phase A: Months 1-2 (foundation)
- Phase B: Months 3-4 (throughput)
- Phase C: Months 5-6 (launch readiness)

### Engineering leadership roles

| Role | Phase A tasks | Phase B tasks | Phase C tasks |
|---|---|---|---|
| VP Engineering | Establish architecture governance, staffing model, engineering SLAs | Resolve cross-team conflicts, enforce scope discipline | Run launch command center and on-call readiness |
| Director Core Engines | Finalize core crate boundaries and ownership for geometry/BIM/constraints | Drive integration milestones across engine teams | Sign off on engine stability and known issue policy |
| Director Platform and Reliability | Stand up CI/CD, build cache, release quality gates | Scale perf and reliability programs | Own release readiness checklist and rollback policy |

### Core engine roles

| Role | Phase A tasks | Phase B tasks | Phase C tasks |
|---|---|---|---|
| Principal Geometry Engineer | Define kernel architecture, tolerance strategy, invariant set | Deliver robust booleans/intersections for v1 workflows | Close top geometry defects and certify kernel baseline |
| Principal Constraint Solver Engineer | Design solver architecture and deterministic solve pipeline | Implement constraint classes for v1 feature set | Tune stability/performance for customer models |
| Staff BIM Data Model Engineer | Define BIM schema, transaction model, and migration strategy | Implement entity graph and dependency relationships | Lock schema versioning and compatibility rules |
| Senior Rust Systems Engineer | Implement core services and shared infra crates | Support integration across desktop/cloud boundaries | Remove high-risk technical debt before launch |

### Rendering, GPU, and desktop roles

| Role | Phase A tasks | Phase B tasks | Phase C tasks |
|---|---|---|---|
| Staff Rendering Engineer | Design rendering architecture and scene graph contracts | Implement selection/highlight and interaction rendering | Optimize frame-time hotspots and visual correctness |
| Senior GPU Engineer | Build Vulkan/Metal abstraction and shader pipeline | Optimize memory bandwidth and shader compilation | Harden GPU fallback behavior and crash diagnostics |
| Senior Desktop Engineer | Build app shell, command bus integration, persistence | Implement core modeling workflow UX plumbing | Stabilize file/session recovery and update path |
| UI Framework Engineer | Implement tool-state architecture and command framework | Improve panel/docking and undo/redo behavior | Final UX latency polish and workflow consistency |

### Interop and data exchange roles

| Role | Phase A tasks | Phase B tasks | Phase C tasks |
|---|---|---|---|
| Senior IFC Interop Engineer | Define IFC mapping strategy and fidelity benchmarks | Implement import/export core flows and diagnostics | Achieve target round-trip accuracy and triage failures |
| DWG Interop Engineer | Establish legal-safe bridge architecture and boundaries | Implement selected DWG pathways for v1 scope | Improve edge-case handling and conversion telemetry |
| Import/Export Pipeline Engineer | Build conversion pipeline orchestration and job model | Add validation, retries, and user-facing diagnostics | Harden reliability and throughput under production load |

### Cloud, auth, collaboration roles

| Role | Phase A tasks | Phase B tasks | Phase C tasks |
|---|---|---|---|
| Cloud Services Engineer | Build project/version service contracts and API skeleton | Implement model versioning and comment services | Harden API reliability, observability, and scaling |
| Authentication Engineer | Implement OIDC login, RBAC model, tenancy primitives | Integrate auth across desktop and cloud services | Complete security review and incident runbooks |
| Collaboration Engineer | Implement async collaboration states and review model | Add conflict awareness and notification workflows | Stabilize collaboration UX and reliability metrics |

### SDK, AI, performance, and platform roles

| Role | Phase A tasks | Phase B tasks | Phase C tasks |
|---|---|---|---|
| Plugin SDK Engineer | Define SDK API surface and extension lifecycle | Build SDK beta APIs and sample plugins | Lock SDK beta contracts and migration notes |
| AI Features Engineer | Build AI command abstraction and safe execution hooks | Implement assisted workflows and model checks | Tune quality and safety with telemetry feedback |
| Staff Developer Platform Engineer | Deliver one-command dev setup and local environments | Improve developer golden paths and internal tooling | Reduce lead time and CI friction for all teams |
| Senior Build Systems Engineer | Implement reproducible builds, caching, and CI partitioning | Optimize build times and artifact reliability | Finalize release build pipeline and signing workflow |
| Infrastructure/SRE Engineer | Build cloud environments, secrets, and deployment baseline | Add observability and SLOs for services | Run launch reliability drills and failure simulations |
| Performance Engineer | Define perf budgets and benchmark harness | Track regressions and optimize critical paths | Sign off launch performance thresholds |
| QA Automation Architect | Define end-to-end test strategy and test oracle architecture | Implement workflow automation and regression suites | Certify release quality gates and flaky test controls |
| QA Automation Engineer | Implement test cases for core workflows | Expand coverage for interop/cloud/SDK areas | Execute launch regression and triage cycles |
| Security Engineer (AppSec) | Define secure coding baseline and threat model | Perform security reviews on SDK/auth/cloud | Complete launch security checklist and response plan |

### Product and design roles supporting engineering execution

| Role | Phase A tasks | Phase B tasks | Phase C tasks |
|---|---|---|---|
| Product Manager, Core Modeling | Prioritize v1 scope and acceptance criteria | Drive cross-team milestone decisions and tradeoffs | Own launch scope lock and backlog freeze |
| Product Designer, Pro Tools | Define interaction model for core workflows | Validate usability with design partners | Finalize UX polish and onboarding flows |
| Technical Writer | Create architecture docs and user workflow drafts | Publish SDK and feature documentation | Deliver launch docs, runbooks, and release notes |
| DevRel Engineer, SDK | Build early partner feedback channel | Produce SDK samples and technical content | Support launch partners and ecosystem onboarding |

## Engineering contribution agreements

1. Every role owns a measurable outcome tied to a crate or service boundary.
2. Every change includes tests at the right boundary level.
3. Every P0 incident has post-incident review within 48 hours.
4. Every team contributes to monthly cross-team hardening sprint.

## Definition of done for launch-critical work

1. Functional acceptance criteria passed.
2. Unit, integration, and scenario tests green.
3. Performance budget met with benchmark evidence.
4. Observability and rollback path documented.
5. Documentation and migration notes published.

## First 2 weeks implementation checklist

1. Create crate skeletons and ownership map.
2. Add CODEOWNERS aligned to role matrix.
3. Add CI pipeline with mandatory gates.
4. Publish first 10 ADRs for key architecture choices.
5. Run first architecture council and dependency review.
