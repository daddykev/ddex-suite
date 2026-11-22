import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const GITHUB = 'https://github.com/daddykev/ddex-suite';

// Old docs URLs (the pre-0.5.0 sidebar, the docs pages and the template blog) -> the closest
// current page, so existing links and search results do not end in a 404. The redirect
// plugin writes a small HTML page at each old path.
const REDIRECTS: Record<string, string[]> = {
  '/docs/intro': ['/docs', '/docs/use-cases', '/docs/guides', '/docs/guides/reference/patterns',
    '/docs/guides/migrating-from-xml', '/docs/guides/migration/from-xml',
    '/docs/guides/migration/from-other-tools', '/docs/guides/migration/legacy-systems',
    '/docs/guides/integration/automation', '/docs/guides/integration/database',
    '/docs/guides/integration/monitoring', '/docs/guides/integration/rest-api',
    '/docs/guides/integration/testing'],
  '/docs/installation': ['/docs/getting-started', '/docs/parser/installation',
    '/docs/builder/installation', '/docs/guides/troubleshooting/deployment'],
  '/docs/quick-start-parsing': ['/docs/parser', '/docs/parser/overview', '/docs/parser/quick-start',
    '/docs/parser/advanced-usage', '/docs/examples', '/docs/examples/parse-ern',
    '/docs/guides/parser/basic-parsing', '/docs/guides/parser/error-handling'],
  '/docs/quick-start-building': ['/docs/builder', '/docs/builder/overview', '/docs/builder/quick-start',
    '/docs/builder/advanced-usage', '/docs/examples/build-new-release', '/docs/examples/round-trip',
    '/docs/examples/batch-processing', '/docs/guides/builder/from-scratch',
    '/docs/guides/builder/batch-processing', '/docs/guides/round-trip-fidelity'],
  '/docs/parse-output': ['/docs/parser/api-reference', '/docs/api/parser', '/docs/api/parser/rust',
    '/docs/api/parser/typescript', '/docs/api/parser/python', '/docs/api/parser/types',
    '/docs/api/models', '/docs/api/models/datetime', '/docs/api/models/deal',
    '/docs/api/models/errors', '/docs/api/models/identifiers', '/docs/api/models/message',
    '/docs/api/models/party', '/docs/api/models/release', '/docs/api/models/resource',
    '/docs/api/models/technical', '/docs/api/models/territories',
    '/docs/guides/reference/territory-codes', '/docs/guides/advanced/extensions'],
  '/docs/build-request': ['/docs/builder/api-reference', '/docs/builder/canonicalization',
    '/docs/api/builder', '/docs/api/builder/rust', '/docs/api/builder/typescript',
    '/docs/api/builder/python', '/docs/api/builder/types', '/docs/api/builder/validation',
    '/docs/guides/builder/deterministic', '/docs/guides/builder/validation',
    '/docs/guides/error-handling', '/docs/guides/troubleshooting/validation-errors'],
  '/docs/ern-versions': ['/docs/guides/reference/schema-versions',
    '/docs/guides/reference/dsp-requirements', '/docs/guides/dsp-integration'],
  '/docs/python-dataframes': ['/docs/examples/python-dataframes', '/docs/guides/parser/dataframes',
    '/docs/guides/python-workflows'],
  '/docs/security': ['/docs/guides/advanced/security', '/docs/guides/advanced/validation-rules',
    '/docs/guides/validation-workflow'],
  '/docs/performance': ['/docs/guides/parser/performance', '/docs/guides/parser/large-files',
    '/docs/guides/reference/benchmarks', '/docs/guides/performance-tuning',
    '/docs/guides/advanced/memory', '/docs/guides/advanced/performance-monitoring',
    '/docs/guides/troubleshooting/performance', '/docs/guides/troubleshooting/memory',
    '/docs/api/parser/memory-management'],
  '/docs/migration': ['/docs/guides/migration/version-upgrades'],
  '/docs/known-issues': ['/docs/builder/presets', '/docs/api/builder/presets',
    '/docs/guides/builder/partner-outputs', '/docs/api/parser/streaming',
    '/docs/guides/streaming-large-files', '/docs/guides/troubleshooting/common-issues'],
  '/docs/api-reference': ['/docs/api', '/docs/api/typedoc'],
  '/': ['/blog', '/blog/archive', '/blog/authors', '/blog/tags', '/blog/first-blog-post',
    '/blog/long-blog-post', '/blog/mdx-blog-post', '/blog/welcome', '/markdown-page'],
};

const config: Config = {
  title: 'DDEX Suite',
  tagline: 'Parse and build DDEX ERN XML in Rust, Node.js and Python',
  favicon: 'img/favicon.ico',
  url: 'https://ddex-suite.org',
  baseUrl: '/',

  organizationName: 'daddykev',
  projectName: 'ddex-suite',

  // Every internal link must resolve: a broken link, Markdown link or anchor fails the build.
  onBrokenLinks: 'throw',
  onBrokenAnchors: 'throw',
  onDuplicateRoutes: 'throw',
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'throw',
      onBrokenMarkdownImages: 'throw',
    },
  },

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
          editUrl: `${GITHUB}/tree/main/website/`,
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
        gtag: {
          trackingID: 'G-H5TSB2963K',
          anonymizeIP: true,
        },
      } satisfies Preset.Options,
    ],
  ],

  plugins: [
    [
      '@docusaurus/plugin-client-redirects',
      {
        redirects: Object.entries(REDIRECTS).map(([to, from]) => ({to, from})),
      },
    ],
  ],

  themeConfig: {
    navbar: {
      title: 'DDEX Suite',
      logo: {
        alt: 'DDEX Suite logo',
        src: 'img/logo.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docs',
          position: 'left',
          label: 'Docs',
        },
        {to: '/playground', label: 'Playground', position: 'left'},
        {href: GITHUB, label: 'GitHub', position: 'right'},
        {href: `${GITHUB}/issues`, label: 'Issues', position: 'right'},
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {label: 'Introduction', to: '/docs/intro'},
            {label: 'Installation', to: '/docs/installation'},
            {label: 'Parse output reference', to: '/docs/parse-output'},
            {label: 'Build request reference', to: '/docs/build-request'},
            {label: 'Known issues and roadmap', to: '/docs/known-issues'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'GitHub', href: GITHUB},
            {label: 'Issues', href: `${GITHUB}/issues`},
            {label: 'Changelog', href: `${GITHUB}/blob/main/CHANGELOG.md`},
            {label: 'DDEX Workbench', href: 'https://ddex-workbench.org'},
          ],
        },
        {
          title: 'Packages',
          items: [
            {label: 'npm: ddex-parser', href: 'https://www.npmjs.com/package/ddex-parser'},
            {label: 'npm: ddex-builder', href: 'https://www.npmjs.com/package/ddex-builder'},
            {label: 'PyPI: ddex-parser', href: 'https://pypi.org/project/ddex-parser/'},
            {label: 'PyPI: ddex-builder', href: 'https://pypi.org/project/ddex-builder/'},
            {label: 'crates.io', href: 'https://crates.io/crates/ddex-parser'},
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} DDEX Suite. Open source under the MIT License.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'python', 'bash', 'json', 'toml'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
