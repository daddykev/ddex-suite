import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const sidebars: SidebarsConfig = {
  // Main documentation sidebar with logical organization
  tutorialSidebar: [
    'intro',
    {
      type: 'category',
      label: 'Getting Started',
      items: [
        'getting-started/index',
      ],
    },
    {
      type: 'category',
      label: 'DDEX Parser',
      items: [
        'parser/index',
      ],
    },
    {
      type: 'category',
      label: 'DDEX Builder',
      items: [
        'builder/index',
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      items: [
        'api/index',
      ],
    },
    {
      type: 'category',
      label: 'Examples',
      items: [
        'examples/index',
      ],
    },
    {
      type: 'category',
      label: 'Guides',
      items: [
        'guides/index',
      ],
    },
  ],
};

export default sidebars;
