import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'DDEX Suite Documentation',
  tagline: 'Parse → Modify → Build DDEX metadata with perfect fidelity',
  favicon: 'img/favicon.ico',
  url: 'https://ddex-suite.org',
  baseUrl: '/',
  
  organizationName: 'daddykev',
  projectName: 'ddex-suite',
  
  onBrokenLinks: 'warn',
  onBrokenMarkdownLinks: 'warn',
  
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/daddykev/ddex-suite/tree/main/website/',
          remarkPlugins: [
            require('remark-math'),
          ],
          rehypePlugins: [
            require('rehype-katex'),
          ],
        },
        theme: {
          customCss: './src/css/custom.css',
        },
        gtag: {
          trackingID: 'G-H5TSB2963K', // Add your Google Analytics ID
          anonymizeIP: true,
        },
      } satisfies Preset.Options,
    ],
  ],

  // plugins: [
  //   [
  //     '@docusaurus/plugin-content-docs',
  //     {
  //       id: 'parser',
  //       path: '../packages/ddex-parser/docs',
  //       routeBasePath: 'parser',
  //       sidebarPath: './sidebarsParser.ts',
  //       editUrl: 'https://github.com/daddykev/ddex-suite/tree/main/packages/ddex-parser/',
  //     },
  //   ],
  //   [
  //     '@docusaurus/plugin-content-docs',
  //     {
  //       id: 'builder',
  //       path: '../packages/ddex-builder/docs',
  //       routeBasePath: 'builder',
  //       sidebarPath: './sidebarsBuilder.ts',
  //       editUrl: 'https://github.com/daddykev/ddex-suite/tree/main/packages/ddex-builder/',
  //     },
  //   ],
  // ],

  themeConfig: {
    image: 'img/ddex-suite-social.jpg',
    navbar: {
      title: 'DDEX Suite',
      logo: {
        alt: 'DDEX Suite Logo',
        src: 'img/logo.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'tutorialSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          to: '/docs/parser/',
          label: 'Parser',
          position: 'left',
        },
        {
          to: '/docs/builder/',
          label: 'Builder',
          position: 'left',
        },
        {to: '/playground', label: 'Playground', position: 'left'},
        {
          type: 'docsVersionDropdown',
          position: 'right',
        },
        {
          href: 'https://github.com/daddykev/ddex-suite',
          label: 'GitHub',
          position: 'right',
        },
        {
          href: 'https://ddex-workbench.org',
          label: 'DDEX Workbench',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Getting Started',
              to: '/docs/intro',
            },
            {
              label: 'Parser API',
              to: '/docs/parser/',
            },
            {
              label: 'Builder API',
              to: '/docs/builder/',
            },
            {
              label: 'API Reference',
              to: '/docs/api/',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/daddykev/ddex-suite',
            },
            {
              label: 'Stack Overflow',
              href: 'https://stackoverflow.com/questions/tagged/ddex',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'DDEX Workbench',
              href: 'https://ddex-workbench.org',
            },
            {
              label: 'npm',
              href: 'https://www.npmjs.com/~daddykev',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Kevin Marques Moo. Open-source under MIT License.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'python', 'bash', 'json'],
    },
    algolia: {
      // Apply for free Algolia DocSearch
      appId: 'YOUR_APP_ID',
      apiKey: 'YOUR_API_KEY',
      indexName: 'ddex-suite',
    },
  } satisfies Preset.ThemeConfig,
  
  stylesheets: [
    {
      href: 'https://cdn.jsdelivr.net/npm/katex@0.13.24/dist/katex.min.css',
      type: 'text/css',
      integrity: 'sha384-odtC+0UGzzFL/6PNoE8rX/SPcQDXBJ+uRepguP4QkPCm2LBxH3FA3y+fKSiJ+AmM',
      crossorigin: 'anonymous',
    },
  ],
};

export default config;