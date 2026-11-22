import React, { useState, useEffect } from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import CodeBlock from '@theme/CodeBlock';
import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
  faFileAlt,
  faPencilAlt,
  faWrench,
  faRocket,
  faBolt,
  faBox,
  faSave,
  faTrophy,
  faSearch,
  faBullseye,
  faTag,
  faCheckCircle,
  faChartBar,
  faExclamationTriangle,
  faWater,
  faHeart,
  faGlobe,
  faIndustry,
  faChartLine,
  faSync,
  faTimesCircle
} from '@fortawesome/free-solid-svg-icons';
import { faPython } from '@fortawesome/free-brands-svg-icons';
import styles from './index.module.css';

function AnimatedWorkflowStep({ isActive, title, icon, description }) {
  return (
    <div className={clsx(styles.workflowStep, { [styles.workflowStepActive]: isActive })}>
      <div className={styles.workflowStepIcon}>
        {icon}
      </div>
      <div className={styles.workflowStepContent}>
        <h3>{title}</h3>
        <p>{description}</p>
      </div>
    </div>
  );
}

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  const [currentStep, setCurrentStep] = useState(0);
  const steps = [
    { title: 'Parse', icon: <FontAwesomeIcon icon={faFileAlt} />, description: 'Transform DDEX XML into structured data' },
    { title: 'Modify', icon: <FontAwesomeIcon icon={faPencilAlt} />, description: 'Update metadata with clean, typed objects' },
    { title: 'Build', icon: <FontAwesomeIcon icon={faWrench} />, description: 'Generate deterministic, compliant XML' }
  ];

  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentStep((prev) => (prev + 1) % steps.length);
    }, 2000);
    return () => clearInterval(interval);
  }, [steps.length]);

  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <h1 className="hero__title">DDEX Suite</h1>
        <p className="hero__subtitle">{siteConfig.tagline}</p>

        {/* Animated workflow */}
        <div className={styles.workflowAnimation}>
          {steps.map((step, index) => (
            <React.Fragment key={index}>
              <AnimatedWorkflowStep
                isActive={currentStep === index}
                title={step.title}
                icon={step.icon}
                description={step.description}
              />
              {index < steps.length - 1 && (
                <div className={styles.workflowArrow}>
                  →
                </div>
              )}
            </React.Fragment>
          ))}
        </div>

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
            Try Playground <FontAwesomeIcon icon={faRocket} className={styles.buttonIcon} />
          </Link>
        </div>
      </div>
    </header>
  );
}

function PerformanceBenchmarks() {
  const benchmarks = [
    { label: 'Parse a typical album', value: '3.4 ms (ERN 4.3, 75 KB)', icon: <FontAwesomeIcon icon={faRocket} />, color: 'var(--ifm-color-primary)' },
    { label: 'Parser throughput', value: '20-24 MiB/s (real ERN 4.3)', icon: <FontAwesomeIcon icon={faBolt} />, color: 'var(--ifm-color-primary)' },
    { label: 'Build a 12-track album', value: '11.4 ms (ERN 4.3)', icon: <FontAwesomeIcon icon={faSave} />, color: 'var(--ifm-color-success)' },
    { label: 'WASM Bundle', value: '114KB (77% under target)', icon: <FontAwesomeIcon icon={faBox} />, color: 'var(--ifm-color-primary)' }
  ];

  return (
    <section className={styles.performanceBenchmarks}>
      <div className="container">
        <h2><FontAwesomeIcon icon={faTrophy} /> Performance Benchmarks</h2>
        <div className="row">
          {benchmarks.map((benchmark, index) => (
            <div key={index} className="col col--3">
              <div className={styles.benchmarkCard}>
                <div className={styles.benchmarkIcon} style={{ color: benchmark.color }}>
                  {benchmark.icon}
                </div>
                <div className={styles.benchmarkValue}>{benchmark.value}</div>
                <div className={styles.benchmarkLabel}>{benchmark.label}</div>
              </div>
            </div>
          ))}
        </div>
        <div className={styles.chartContainer}>
          <div className={styles.performanceChart}>
            <h3>Measured parse times (M1 Pro, criterion)</h3>
            <div className={styles.chartBars}>
              <div className={styles.chartBar} style={{ height: '10%' }}>
                <span>10KB<br/>0.5ms</span>
              </div>
              <div className={styles.chartBar} style={{ height: '20%' }}>
                <span>75KB<br/>3.4ms</span>
              </div>
              <div className={styles.chartBar} style={{ height: '30%' }}>
                <span>330KB<br/>14ms</span>
              </div>
              <div className={styles.chartBar} style={{ height: '100%' }}>
                <span>10MB<br/>441ms</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function CodeExamples() {
  const typeScriptExample = `// Parse DDEX
const { DdexParser } = require('ddex-parser');
const parser = new DdexParser();
const data = await parser.parse(xmlString);

// Build DDEX
const { DdexBuilder } = require('ddex-builder');
const builder = new DdexBuilder();
builder.applyPreset('spotify_album'); // optional
const xml = await builder.build(request);

// Batch process
const { batchBuild } = require('ddex-builder');
const results = await batchBuild(jsonStringArray);`;

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

  const cliExample = `# Parse DDEX file to JSON
ddex-parser parse release.xml > release.json

# Validate and analyze
ddex-parser validate release.xml
ddex-parser analyze release.xml

# Build from JSON
ddex-builder build release.json release.xml

# Use presets for different platforms
ddex-builder build --preset youtube_album release.json`;

  const rustExample = `use ddex_parser::DDEXParser;
use ddex_builder::DDEXBuilder;
use std::fs;

// Parse DDEX XML to structured data
let parser = DDEXParser::new();
let xml_content = fs::read_to_string("release.xml")?;
let result = parser.parse(&xml_content)?;

// Access clean, typed data
println!("Release title: {}", result.flat.releases[0].title);
println!("Artist name: {}", result.flat.sound_recordings[0].artist);

// Modify the data
result.flat.releases[0].title = "Remastered Edition".to_string();
result.flat.deals[0].territories.extend_from_slice(&["US", "CA", "GB"]);

// Build back to deterministic XML
let builder = DDEXBuilder::new();
let new_xml = builder.build(&result.to_build_request())?;`;

  return (
    <section className={styles.codeExamples}>
      <div className="container">
        <h2>Parse → Modify → Build in Four Languages</h2>
        <Tabs>
          <TabItem value="typescript" label="TypeScript" default>
            <CodeBlock language="typescript">{typeScriptExample}</CodeBlock>
            <div className={styles.installCommand}>
              <CodeBlock language="bash">npm install ddex-parser ddex-builder</CodeBlock>
            </div>
          </TabItem>
          <TabItem value="python" label="Python">
            <CodeBlock language="python">{pythonExample}</CodeBlock>
            <div className={styles.installCommand}>
              <CodeBlock language="bash">pip install ddex-parser ddex-builder</CodeBlock>
            </div>
          </TabItem>
          <TabItem value="cli" label="CLI">
            <CodeBlock language="bash">{cliExample}</CodeBlock>
            <div className={styles.installCommand}>
              <CodeBlock language="bash">cargo install ddex-parser ddex-builder</CodeBlock>
            </div>
          </TabItem>
          <TabItem value="rust" label="Rust">
            <CodeBlock language="rust">{rustExample}</CodeBlock>
            <div className={styles.installCommand}>
              <CodeBlock language="bash">cargo add ddex-parser ddex-builder</CodeBlock>
            </div>
          </TabItem>
        </Tabs>
      </div>
    </section>
  );
}

function FeatureComparison() {
  const parserFeatures = [
    { icon: <FontAwesomeIcon icon={faFileAlt} />, title: 'Multi-Version Support', desc: 'ERN 3.8.2, 4.2, and 4.3' },
    { icon: <FontAwesomeIcon icon={faSearch} />, title: 'Dual Representations', desc: 'Graph and flattened data models' },
    { icon: <FontAwesomeIcon icon={faPython} />, title: 'DataFrame Integration', desc: 'Native pandas/polars support' },
    { icon: <FontAwesomeIcon icon={faWater} />, title: 'Streaming Parser', desc: 'Handle GB+ files efficiently' },
    { icon: <FontAwesomeIcon icon={faExclamationTriangle} />, title: 'Detailed Errors', desc: 'Precise validation feedback' },
  ];

  const builderFeatures = [
    { icon: <FontAwesomeIcon icon={faBullseye} />, title: 'Deterministic Output', desc: 'Byte-perfect reproducibility' },
    { icon: <FontAwesomeIcon icon={faTag} />, title: 'Platform Presets', desc: 'Spotify, Apple, YouTube ready' },
    { icon: <FontAwesomeIcon icon={faCheckCircle} />, title: 'Preflight Validation', desc: 'Catch errors before building' },
    { icon: <FontAwesomeIcon icon={faWrench} />, title: 'DB-C14N/1.0', desc: 'Industry-standard canonicalization' },
    { icon: <FontAwesomeIcon icon={faChartBar} />, title: 'DataFrame Builder', desc: 'Build from structured data' },
  ];

  return (
    <section className={styles.featureComparison}>
      <div className="container">
        <h2>Parser vs Builder: Complementary Powerhouses</h2>
        <div className="row">
          <div className="col col--6">
            <div className={styles.featureCard} style={{ borderLeft: '4px solid var(--ddex-parser-color)' }}>
              <h3><FontAwesomeIcon icon={faSearch} /> DDEX Parser</h3>
              <p className={styles.featureCardDesc}>Transform DDEX XML into clean, structured data</p>
              <div className={styles.featureGrid}>
                {parserFeatures.map((feature, index) => (
                  <div key={index} className={styles.featureItem}>
                    <span className={styles.featureItemIcon}>{feature.icon}</span>
                    <div>
                      <strong>{feature.title}</strong>
                      <br />
                      <small>{feature.desc}</small>
                    </div>
                  </div>
                ))}
              </div>
              <Link className="button button--primary" to="/docs/parser/">
                Parser Docs →
              </Link>
            </div>
          </div>
          <div className="col col--6">
            <div className={styles.featureCard} style={{ borderLeft: '4px solid var(--ddex-builder-color)' }}>
              <h3><FontAwesomeIcon icon={faWrench} /> DDEX Builder</h3>
              <p className={styles.featureCardDesc}>Generate deterministic, compliant DDEX XML</p>
              <div className={styles.featureGrid}>
                {builderFeatures.map((feature, index) => (
                  <div key={index} className={styles.featureItem}>
                    <span className={styles.featureItemIcon}>{feature.icon}</span>
                    <div>
                      <strong>{feature.title}</strong>
                      <br />
                      <small>{feature.desc}</small>
                    </div>
                  </div>
                ))}
              </div>
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

function WhyDDEXSuite() {
  const comparisons = [
    {
      feature: 'Performance',
      ddexSuite: 'Rust core: a typical album parses in about 3 ms',
      alternatives: 'Slow, memory-intensive parsing',
      icon: <FontAwesomeIcon icon={faRocket} />
    },
    {
      feature: 'Data Fidelity',
      ddexSuite: 'Perfect round-trip guarantee',
      alternatives: 'Data loss during transformations',
      icon: <FontAwesomeIcon icon={faSync} />
    },
    {
      feature: 'Multi-Platform',
      ddexSuite: 'Native Node.js, Python, WASM, CLI',
      alternatives: 'Single language or poor bindings',
      icon: <FontAwesomeIcon icon={faGlobe} />
    },
    {
      feature: 'Developer Experience',
      ddexSuite: 'TypeScript definitions, detailed errors',
      alternatives: 'Poor documentation, cryptic errors',
      icon: <FontAwesomeIcon icon={faHeart} />
    },
    {
      feature: 'Industry Standards',
      ddexSuite: 'DB-C14N/1.0, platform presets',
      alternatives: 'Generic XML tools',
      icon: <FontAwesomeIcon icon={faIndustry} />
    },
    {
      feature: 'Scalability',
      ddexSuite: 'Stream GB+ files with <100MB memory',
      alternatives: 'Memory explosion with large files',
      icon: <FontAwesomeIcon icon={faChartLine} />
    }
  ];

  return (
    <section className={styles.whyDDEXSuite}>
      <div className="container">
        <h2>Why Choose DDEX Suite?</h2>
        <p className={styles.sectionSubtitle}>
          Purpose-built for DDEX processing, not adapted from generic XML tools
        </p>
        <div className={styles.comparisonTable}>
          {comparisons.map((comparison, index) => (
            <div key={index} className={styles.comparisonRow}>
              <div className={styles.comparisonFeature}>
                <span className={styles.comparisonIcon}>{comparison.icon}</span>
                <strong>{comparison.feature}</strong>
              </div>
              <div className={styles.comparisonDDEX}>
                <FontAwesomeIcon icon={faCheckCircle} /> {comparison.ddexSuite}
              </div>
              <div className={styles.comparisonAlts}>
                <FontAwesomeIcon icon={faTimesCircle} /> {comparison.alternatives}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

export default function Home() {
  return (
    <Layout
      title="High-Performance DDEX Processing"
      description="Parse and build DDEX XML with perfect fidelity. Native bindings for Node.js and Python with TypeScript support.">
      <HomepageHeader />
      <main>
        <PerformanceBenchmarks />
        <CodeExamples />
        <FeatureComparison />
        <WhyDDEXSuite />
      </main>
    </Layout>
  );
}
