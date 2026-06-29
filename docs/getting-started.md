# Getting Started

## Local development

1. Install dependencies:

```bash
bun install
```

2. Start docs site locally:

```bash
bun run docs:dev
```

3. Build static site:

```bash
bun run docs:build
```

4. Preview production build:

```bash
bun run docs:preview
```

## Suggested authoring workflow

1. Start from a template in [PRD Template](./templates/product-requirements-document.md) or [API Contract Template](./engineering/api-contract-template.md).
2. Fill metadata sections first (owner, status, target release).
3. Open a pull request with doc changes.
4. Request review from role owners and adjacent teams.
5. Merge to main to publish automatically through GitHub Actions.

## Recommended placeholders to use first

- Product roadmap: [Roadmap Template](./product/roadmap-template.md)
- Feature details: [Feature Spec Template](./templates/feature-spec-template.md)
- Architecture decisions: [RFC Template](./rfc/0000-template.md) and [ADR Template](./adr/0000-template.md)
- Operations readiness: [Incident Runbook Template](./operations/incident-runbook-template.md)
