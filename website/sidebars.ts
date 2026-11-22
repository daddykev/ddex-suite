import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// The sidebar lists every docs page. A page that is not listed here must not exist
// (docs/ holds exactly these files).
const sidebars: SidebarsConfig = {
  docs: [
    'intro',
    'installation',
    'quick-start-parsing',
    'quick-start-building',
    'parse-output',
    'build-request',
    'ern-versions',
    'python-dataframes',
    'security',
    'performance',
    'migration',
    'known-issues',
    'playground',
    'api-reference',
  ],
};

export default sidebars;
