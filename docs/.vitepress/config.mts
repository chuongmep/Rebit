import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Rebit Docs',
  description: 'Product, architecture, and delivery documentation for Rebit',
  lang: 'en-US',
  lastUpdated: true,
  cleanUrls: true,
  themeConfig: {
    logo: '/logo.svg',
    siteTitle: 'Rebit Docs',
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Start', link: '/getting-started' },
      { text: 'Product', link: '/product/roadmap-template' },
      { text: 'Engineering', link: '/engineering/api-contract-template' },
      { text: 'Operations', link: '/operations/incident-runbook-template' },
      { text: 'Templates', link: '/templates/product-requirements-document' }
    ],
    sidebar: [
      {
        text: 'Start Here',
        items: [
          { text: 'Overview', link: '/' },
          { text: 'Getting Started', link: '/getting-started' }
        ]
      },
      {
        text: 'Product Docs',
        items: [
          { text: 'Roadmap Template', link: '/product/roadmap-template' },
          { text: 'Release Notes Template', link: '/product/release-notes-template' },
          { text: 'PRD Template', link: '/templates/product-requirements-document' },
          { text: 'Feature Spec Template', link: '/templates/feature-spec-template' },
          { text: 'Release Plan Template', link: '/templates/release-plan-template' }
        ]
      },
      {
        text: 'Engineering Docs',
        items: [
          { text: 'API Contract Template', link: '/engineering/api-contract-template' },
          { text: 'RFC Template', link: '/rfc/0000-template' },
          { text: 'ADR Template', link: '/adr/0000-template' }
        ]
      },
      {
        text: 'Operations Docs',
        items: [
          { text: 'Incident Runbook Template', link: '/operations/incident-runbook-template' },
          { text: 'Onboarding Runbook', link: '/runbooks/onboarding_engineer' }
        ]
      }
    ],
    socialLinks: [{ icon: 'github', link: 'https://github.com/' }],
    search: {
      provider: 'local'
    },
    outline: {
      label: 'On this page',
      level: [2, 3]
    },
    editLink: {
      pattern: 'https://github.com/<org>/<repo>/edit/main/docs/:path',
      text: 'Edit this page'
    },
    docFooter: {
      prev: 'Previous page',
      next: 'Next page'
    },
    footer: {
      message: 'Built with VitePress for product and engineering teams.',
      copyright: 'Copyright Rebit'
    }
  }
})
