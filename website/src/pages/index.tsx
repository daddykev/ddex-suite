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
        <h1 className="hero__title">DDEX Suite</h1>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.badges}>
          <img src="https://img.shields.io/npm/v/ddex-parser?label=parser&style=flat-square" alt="Parser version" />
          <img src="https://img.shields.io/npm/v/ddex-builder?label=builder&style=flat-square" alt="Builder version" />
          <img src="https://img.shields.io/github/license/daddykev/ddex-suite?style=flat-square" alt="License" />
          <img src="https://img.shields.io/npm/dm/ddex-parser?style=flat-square" alt="Downloads" />
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

function ValueProposition() {
  return (
    <section className={styles.valueProposition}>
      <div className="container">
        <div className="row">
          <div className="col col--6">
            <h2>🎵 Built for the Music Industry</h2>
            <p>
              DDEX Suite transforms complex XML metadata into clean, developer-friendly objects. 
              Process Electronic Release Notifications (ERN) with confidence, knowing your data 
              integrity is preserved through every transformation.
            </p>
            <ul>
              <li><strong>Perfect Fidelity:</strong> Round-trip guarantee preserves all data</li>
              <li><strong>Production Ready:</strong> Used by labels processing millions of releases</li>
              <li><strong>Multi-Language:</strong> Native Node.js, Python, and WASM support</li>
            </ul>
          </div>
          <div className="col col--6">
            <h2>⚡ Performance That Scales</h2>
            <div className={styles.performanceMetrics}>
              <div className={styles.metric}>
                <div className={styles.metricValue}>{'<5ms'}</div>
                <div className={styles.metricLabel}>Parse 10KB files</div>
              </div>
              <div className={styles.metric}>
                <div className={styles.metricValue}>{'<5s'}</div>
                <div className={styles.metricLabel}>Process 100MB files</div>
              </div>
              <div className={styles.metric}>
                <div className={styles.metricValue}>{'<100MB'}</div>
                <div className={styles.metricLabel}>Memory for 1GB+ streams</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function QuickExample() {
  const parserExample = `import { DDEXParser, DDEXBuilder } from 'ddex-suite';

// Parse DDEX XML to structured data
const parser = new DDEXParser();
const result = await parser.parse(xmlContent);

// Access clean, typed data
console.log(result.flat.releases[0].title);
console.log(result.flat.soundRecordings[0].artist);

// Modify the data
result.flat.releases[0].title = "Remastered Edition";
result.flat.deals[0].territories.push("US", "CA", "GB");

// Build back to deterministic XML
const builder = new DDEXBuilder();
const newXml = await builder.build(result.toBuildRequest());`;

  const pythonExample = `from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

# Parse to DataFrame for analysis
parser = DDEXParser()
df = parser.to_dataframe('release.xml')

# Analyze with pandas
print(df.releases.groupby('artist').count())

# Build from DataFrame
builder = DDEXBuilder()
xml = builder.from_dataframe(df, version='4.3')`;

  return (
    <section className={styles.quickExample}>
      <div className="container">
        <h2>Parse → Modify → Build in seconds</h2>
        <div className="row">
          <div className="col col--6">
            <h3>TypeScript / Node.js</h3>
            <CodeBlock language="typescript">{parserExample}</CodeBlock>
            <div className={styles.installCommand}>
              <CodeBlock language="bash">npm install ddex-parser ddex-builder</CodeBlock>
            </div>
          </div>
          <div className="col col--6">
            <h3>Python</h3>
            <CodeBlock language="python">{pythonExample}</CodeBlock>
            <div className={styles.installCommand}>
              <CodeBlock language="bash">pip install ddex-parser ddex-builder</CodeBlock>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function FeatureComparison() {
  return (
    <section className={styles.featureComparison}>
      <div className="container">
        <h2>Parser vs Builder: Two Tools, One Workflow</h2>
        <div className="row">
          <div className="col col--6">
            <div className={styles.featureCard}>
              <h3>🔍 DDEX Parser</h3>
              <p>Transform DDEX XML into clean, structured data</p>
              <ul>
                <li>Parse ERN 3.8.2, 4.2, and 4.3</li>
                <li>Graph and flattened representations</li>
                <li>DataFrame integration for Python</li>
                <li>Streaming support for large files</li>
                <li>Comprehensive error reporting</li>
              </ul>
              <Link className="button button--primary" to="/docs/parser/">
                Parser Docs →
              </Link>
            </div>
          </div>
          <div className="col col--6">
            <div className={styles.featureCard}>
              <h3>🔧 DDEX Builder</h3>
              <p>Generate deterministic, compliant DDEX XML</p>
              <ul>
                <li>Deterministic XML generation</li>
                <li>Partner presets (Spotify, Apple, YouTube)</li>
                <li>Preflight validation with detailed errors</li>
                <li>DB-C14N/1.0 canonicalization</li>
                <li>Build from DataFrames or objects</li>
              </ul>
              <Link className="button button--primary" to="/docs/builder/">
                Builder Docs →
              </Link>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

export default function Home(): JSX.Element {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title="High-Performance DDEX Processing"
      description="Parse and build DDEX XML with perfect fidelity. Native bindings for Node.js and Python with TypeScript support.">
      <HomepageHeader />
      <main>
        <ValueProposition />
        <QuickExample />
        <FeatureComparison />
        <HomepageFeatures />
      </main>
    </Layout>
  );
}