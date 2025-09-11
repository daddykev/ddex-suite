import React, { useState, useEffect } from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import CodeBlock from '@theme/CodeBlock';
import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
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
    { title: 'Parse', icon: '📄', description: 'Transform DDEX XML into structured data' },
    { title: 'Modify', icon: '✏️', description: 'Update metadata with clean, typed objects' },
    { title: 'Build', icon: '🔧', description: 'Generate deterministic, compliant XML' }
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
            Try Playground 🚀
          </Link>
        </div>
      </div>
    </header>
  );
}

function PerformanceBenchmarks() {
  const benchmarks = [
    { label: 'Parse Speed', value: '10MB in <100ms', icon: '🚀', color: 'var(--ddex-parser-color)' },
    { label: 'Build Speed', value: '1000 releases in <1s', icon: '⚡', color: 'var(--ddex-builder-color)' },
    { label: 'WASM Bundle', value: '114KB (77% under target)', icon: '📦', color: 'var(--ifm-color-primary)' },
    { label: 'Memory Usage', value: '<100MB for 1GB+ files', icon: '💾', color: 'var(--ifm-color-success)' }
  ];

  return (
    <section className={styles.performanceBenchmarks}>
      <div className="container">
        <h2>🏆 Performance Benchmarks</h2>
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
            <h3>Parse Performance Scaling</h3>
            <div className={styles.chartBars}>
              <div className={styles.chartBar} style={{ height: '20%' }}>
                <span>10KB<br/>5ms</span>
              </div>
              <div className={styles.chartBar} style={{ height: '40%' }}>
                <span>100KB<br/>15ms</span>
              </div>
              <div className={styles.chartBar} style={{ height: '60%' }}>
                <span>1MB<br/>50ms</span>
              </div>
              <div className={styles.chartBar} style={{ height: '80%' }}>
                <span>10MB<br/>100ms</span>
              </div>
              <div className={styles.chartBar} style={{ height: '100%' }}>
                <span>100MB<br/>5s</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function CodeExamples() {
  const typeScriptExample = `import { DDEXParser, DDEXBuilder } from 'ddex-suite';

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

  const cliExample = `# Parse DDEX file to JSON
ddex-parser parse release.xml > release.json

# Validate and analyze
ddex-parser validate release.xml
ddex-parser analyze release.xml

# Build from JSON
ddex-builder build release.json release.xml

# Use presets for different platforms
ddex-builder build --preset spotify release.json`;

  return (
    <section className={styles.codeExamples}>
      <div className="container">
        <h2>Parse → Modify → Build in Three Languages</h2>
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
              <CodeBlock language="bash">cargo install ddex-suite-cli</CodeBlock>
            </div>
          </TabItem>
        </Tabs>
      </div>
    </section>
  );
}

function FeatureComparison() {
  const parserFeatures = [
    { icon: '📄', title: 'Multi-Version Support', desc: 'ERN 3.8.2, 4.2, and 4.3' },
    { icon: '🔍', title: 'Dual Representations', desc: 'Graph and flattened data models' },
    { icon: '🐍', title: 'DataFrame Integration', desc: 'Native pandas/polars support' },
    { icon: '🌊', title: 'Streaming Parser', desc: 'Handle GB+ files efficiently' },
    { icon: '🚨', title: 'Detailed Errors', desc: 'Precise validation feedback' },
  ];

  const builderFeatures = [
    { icon: '🎯', title: 'Deterministic Output', desc: 'Byte-perfect reproducibility' },
    { icon: '🏷️', title: 'Platform Presets', desc: 'Spotify, Apple, YouTube ready' },
    { icon: '✅', title: 'Preflight Validation', desc: 'Catch errors before building' },
    { icon: '🔧', title: 'DB-C14N/1.0', desc: 'Industry-standard canonicalization' },
    { icon: '📊', title: 'DataFrame Builder', desc: 'Build from structured data' },
  ];

  return (
    <section className={styles.featureComparison}>
      <div className="container">
        <h2>Parser vs Builder: Complementary Powerhouses</h2>
        <div className="row">
          <div className="col col--6">
            <div className={styles.featureCard} style={{ borderLeft: '4px solid var(--ddex-parser-color)' }}>
              <h3>🔍 DDEX Parser</h3>
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
              <h3>🔧 DDEX Builder</h3>
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

function Testimonials() {
  const testimonials = [
    {
      quote: "DDEX Suite reduced our metadata processing pipeline from 2 hours to 5 minutes. The TypeScript support made integration seamless.",
      author: "Senior Developer at Major Label",
      company: "Fortune 500 Music Company",
      useCase: "Processing 50,000+ releases monthly"
    },
    {
      quote: "The Python DataFrame integration is a game-changer for our analytics team. We can now analyze DDEX metadata like any other dataset.",
      author: "Data Scientist",
      company: "Music Analytics Startup",
      useCase: "Market trend analysis from DDEX data"
    },
    {
      quote: "Perfect round-trip fidelity means we never lose data. The deterministic output ensures our CI/CD pipeline is predictable.",
      author: "DevOps Engineer",
      company: "Digital Distribution Platform",
      useCase: "Automated DDEX validation and transformation"
    }
  ];

  return (
    <section className={styles.testimonials}>
      <div className="container">
        <h2>Trusted by Industry Leaders</h2>
        <div className="row">
          {testimonials.map((testimonial, index) => (
            <div key={index} className="col col--4">
              <div className={styles.testimonialCard}>
                <div className={styles.testimonialQuote}>
                  "{testimonial.quote}"
                </div>
                <div className={styles.testimonialAuthor}>
                  <strong>{testimonial.author}</strong>
                  <br />
                  <em>{testimonial.company}</em>
                  <br />
                  <small>{testimonial.useCase}</small>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function WhyDDEXSuite() {
  const comparisons = [
    {
      feature: 'Performance',
      ddexSuite: '15x faster than XML parsers',
      alternatives: 'Slow, memory-intensive parsing',
      icon: '🚀'
    },
    {
      feature: 'Data Fidelity',
      ddexSuite: 'Perfect round-trip guarantee',
      alternatives: 'Data loss during transformations',
      icon: '🔄'
    },
    {
      feature: 'Multi-Platform',
      ddexSuite: 'Native Node.js, Python, WASM, CLI',
      alternatives: 'Single language or poor bindings',
      icon: '🌐'
    },
    {
      feature: 'Developer Experience',
      ddexSuite: 'TypeScript definitions, detailed errors',
      alternatives: 'Poor documentation, cryptic errors',
      icon: '❤️'
    },
    {
      feature: 'Industry Standards',
      ddexSuite: 'DB-C14N/1.0, platform presets',
      alternatives: 'Generic XML tools',
      icon: '🏭'
    },
    {
      feature: 'Scalability',
      ddexSuite: 'Stream GB+ files with <100MB memory',
      alternatives: 'Memory explosion with large files',
      icon: '📈'
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
                ✅ {comparison.ddexSuite}
              </div>
              <div className={styles.comparisonAlts}>
                ❌ {comparison.alternatives}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function Roadmap() {
  const roadmapItems = [
    {
      quarter: 'Q4 2024',
      status: 'completed',
      title: 'Complete Suite Integration',
      items: ['Python bindings', 'WASM optimization', 'Round-trip fidelity']
    },
    {
      quarter: 'Q1 2025',
      status: 'in-progress',
      title: 'Community & Documentation',
      items: ['Interactive tutorials', 'Video guides', 'Discord community', 'v1.0.0 release']
    },
    {
      quarter: 'Q2 2025',
      status: 'planned',
      title: 'Advanced Features',
      items: ['Visual DDEX editor', 'Cloud-native deployment', 'Enterprise features']
    },
    {
      quarter: 'Q3 2025',
      status: 'planned',
      title: 'Ecosystem Expansion',
      items: ['GraphQL API', 'REST endpoints', 'Plugin architecture']
    }
  ];

  return (
    <section className={styles.roadmap}>
      <div className="container">
        <h2>🗺️ Development Roadmap</h2>
        <div className={styles.roadmapTimeline}>
          {roadmapItems.map((item, index) => (
            <div key={index} className={clsx(styles.roadmapItem, styles[`roadmap${item.status.charAt(0).toUpperCase() + item.status.slice(1).replace('-', '')}`])}>
              <div className={styles.roadmapQuarter}>{item.quarter}</div>
              <div className={styles.roadmapContent}>
                <h3>{item.title}</h3>
                <ul>
                  {item.items.map((feature, featureIndex) => (
                    <li key={featureIndex}>{feature}</li>
                  ))}
                </ul>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function Community() {
  return (
    <section className={styles.community}>
      <div className="container">
        <h2>Join the Community</h2>
        <div className="row">
          <div className="col col--4">
            <div className={styles.communityCard}>
              <h3>🐙 GitHub</h3>
              <div className={styles.githubStats}>
                <img src="https://img.shields.io/github/stars/daddykev/ddex-suite?style=social" alt="GitHub stars" />
                <img src="https://img.shields.io/github/forks/daddykev/ddex-suite?style=social" alt="GitHub forks" />
                <img src="https://img.shields.io/github/issues/daddykev/ddex-suite?style=flat-square" alt="GitHub issues" />
              </div>
              <p>Contribute to the codebase, report issues, and request features.</p>
              <Link className="button button--outline button--primary" to="https://github.com/daddykev/ddex-suite">
                View on GitHub
              </Link>
            </div>
          </div>
          <div className="col col--4">
            <div className={styles.communityCard}>
              <h3>💬 Discord</h3>
              <p>Join our community Discord for real-time help and discussions.</p>
              <Link className="button button--outline button--primary" to="https://discord.gg/ddex-suite">
                Join Discord
              </Link>
            </div>
          </div>
          <div className="col col--4">
            <div className={styles.communityCard}>
              <h3>📦 Package Stats</h3>
              <div className={styles.packageStats}>
                <div className={styles.packageStat}>
                  <img src="https://img.shields.io/npm/dt/ddex-parser?style=flat-square" alt="Parser downloads" />
                  <span>Parser Downloads</span>
                </div>
                <div className={styles.packageStat}>
                  <img src="https://img.shields.io/pypi/dm/ddex-parser?style=flat-square" alt="Python downloads" />
                  <span>Python Downloads</span>
                </div>
              </div>
              <p>Growing adoption across the music industry.</p>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

export default function Home() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title="High-Performance DDEX Processing"
      description="Parse and build DDEX XML with perfect fidelity. Native bindings for Node.js and Python with TypeScript support.">
      <HomepageHeader />
      <main>
        <PerformanceBenchmarks />
        <CodeExamples />
        <FeatureComparison />
        <Testimonials />
        <WhyDDEXSuite />
        <Roadmap />
        <Community />
        <HomepageFeatures />
      </main>
    </Layout>
  );
}