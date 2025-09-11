import React from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import CodeBlock from '@theme/CodeBlock';
import styles from './index.module.css';

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <h1 className="hero__title">{siteConfig.title}</h1>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.badges}>
          <img src="https://img.shields.io/npm/v/ddex-parser?label=parser" />
          <img src="https://img.shields.io/npm/v/ddex-builder?label=builder" />
          <img src="https://img.shields.io/github/license/daddykev/ddex-suite" />
        </div>
        <div className={styles.buttons}>
          <Link
            className="button button--secondary button--lg"
            to="/docs/intro">
            Get Started - 5min ⏱️
          </Link>
          <Link
            className="button button--outline button--secondary button--lg margin-left--md"
            to="/playground">
            Try Playground 🚀
          </Link>
        </div>
      </div>
    </header>
  );
}

function QuickExample() {
  const parserExample = `import { DdexParser } from 'ddex-parser';
import { DdexBuilder } from 'ddex-builder';

// Parse DDEX XML
const parser = new DdexParser();
const ern = await parser.parse(xmlContent);

// Modify the data
ern.releases[0].title = "Modified Title";

// Build back to XML
const builder = new DdexBuilder();
const xml = await builder.build(ern);`;

  return (
    <section className={styles.quickExample}>
      <div className="container">
        <h2>Parse → Modify → Build in seconds</h2>
        <CodeBlock language="typescript">{parserExample}</CodeBlock>
      </div>
    </section>
  );
}

export default function Home(): JSX.Element {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title={`High-Performance DDEX Processing`}
      description="Parse and build DDEX XML with perfect fidelity. Native bindings for Node.js and Python.">
      <HomepageHeader />
      <main>
        <QuickExample />
        <HomepageFeatures />
      </main>
    </Layout>
  );
}