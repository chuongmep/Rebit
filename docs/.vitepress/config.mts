import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Rebit Docs',
  description: 'Product, architecture, and delivery documentation for Rebit',
  lastUpdated: true,
  cleanUrls: true,
  themeConfig: {
    logo: '/logo.svg',
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Getting Started', link: '/getting-started' },
      { text: 'Templates', link: '/templates/product-requirements-document' },
      { text: 'RFC', link: '/rfc/0000-template' },
      { text: 'ADR', link: '/adr/0000-template' },
      { text: 'Runbooks', link: '/runbooks/onboarding_engineer' }
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
        text: 'Product Templates',
        items: [
          { text: 'PRD Template', link: '/templates/product-requirements-document' },
          { text: 'Feature Spec Template', link: '/templates/feature-spec-template' },
          { text: 'Release Plan Template', link: '/templates/release-plan-template' }
        ]
      },
      {
        text: 'Architecture',
        items: [
          { text: 'RFC Template', link: '/rfc/0000-template' },
          { text: 'ADR Template', link: '/adr/0000-template' }
        ]
      },
      {
        text: 'Operations',
        items: [
          { text: 'Onboarding Runbook', link: '/runbooks/onboarding_engineer' }
        ]
      }
    ],
    socialLinks: [{ icon: 'github', link: 'https://github.com/' }],
    search: {
      provider: 'local'
    },
    footer: {
      message: 'Internal documentation for Rebit teams.',
      copyright: 'Copyright Rebit'
    }
  }
})
