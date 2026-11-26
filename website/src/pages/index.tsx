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
  faSave,
  faTrophy,
  faLayerGroup,
  faEraser,
  faCheckCircle,
  faSync,
  faShieldAlt,
  faTable,
  faDatabase,
} from '@fortawesome/free-solid-svg-icons';
import styles from './index.module.css';

// The code tabs show these files verbatim; CI runs each of them (scripts/readme_examples.py,
// job "Website docs examples") against an official ERN 4.3 sample.
import nodeExample from '!!raw-loader!../examples/landing.js';
import pythonExample from '!!raw-loader!../examples/landing.py';
import rustExample from '!!raw-loader!../examples/landing.rs';

function AnimatedWorkflowStep({ isActive, title, icon, description }) {
  return (
    <div className={clsx(styles.workflowStep, { [styles.workflowStepActive]: isActive })}>
      <div className={styles.workflowStepIcon}>{icon}</div>
      <div className={styles.workflowStepContent}>
        <h3>{title}</h3>
        <p>{description}</p>
      </div>
    </div>
  );
}

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  const [currentStep, setCurrentStep] = useState(0);
  const steps = [
    { title: 'Parse', icon: <FontAwesomeIcon icon={faFileAlt} />, description: 'Read ERN 3.8.x, 4.2 and 4.3 into typed data' },
    { title: 'Modify', icon: <FontAwesomeIcon icon={faPencilAlt} />, description: 'Work with plain objects in your own code' },
    { title: 'Build', icon: <FontAwesomeIcon icon={faWrench} />, description: 'Write schema-valid ERN 4.3, 4.2 or 3.8.2' },
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

        <div className={styles.workflowAnimation}>
          {steps.map((step, index) => (
            <React.Fragment key={step.title}>
              <AnimatedWorkflowStep
                isActive={currentStep === index}
                title={step.title}
                icon={step.icon}
                description={step.description}
              />
              {index < steps.length - 1 && <div className={styles.workflowArrow}>→</div>}
            </React.Fragment>
          ))}
        </div>

        <div className={styles.badges}>
          <img src="https://img.shields.io/npm/v/ddex-parser?label=parser&style=flat-square" alt="ddex-parser version on npm" />
          <img src="https://img.shields.io/npm/v/ddex-builder?label=builder&style=flat-square" alt="ddex-builder version on npm" />
          <img src="https://img.shields.io/github/license/daddykev/ddex-suite?style=flat-square" alt="License" />
        </div>
        <div className={styles.buttons}>
          <Link className="button button--secondary button--lg" to="/docs/intro">
            Get started
          </Link>
          <Link className="button button--outline button--secondary button--lg margin-left--md" to="/playground">
            Try the Playground <FontAwesomeIcon icon={faRocket} className={styles.buttonIcon} />
          </Link>
        </div>
      </div>
    </header>
  );
}

// Measured numbers from dev-docs/PERFORMANCE.md (criterion, 0.5.1, GitHub-hosted runner,
// release profile). Do not add figures that are not in that file.
function PerformanceBenchmarks() {
  const benchmarks = [
    { label: 'Parse a typical album (ERN 4.3, 75 KB)', value: '1.7 ms', icon: faRocket },
    { label: 'Parser throughput on real ERN 4.3', value: '33–47 MiB/s', icon: faBolt },
    { label: 'Parse a ~10 MB catalog', value: '211 ms', icon: faDatabase },
    { label: 'Build a 12-track album (ERN 4.3)', value: '8.3 ms', icon: faSave },
  ];
  // Bar heights on a log scale: 0.1 ms -> 0 %, 1000 ms -> 100 %.
  const bars = [
    { size: '10 KB', ms: 0.27, label: '0.27 ms' },
    { size: '75 KB', ms: 1.7, label: '1.7 ms' },
    { size: '330 KB', ms: 7.43, label: '7.4 ms' },
    { size: '10 MB', ms: 211, label: '211 ms' },
  ];
  const height = (ms: number) => `${Math.round(((Math.log10(ms) + 1) / 4) * 100)}%`;

  return (
    <section className={styles.performanceBenchmarks}>
      <div className="container">
        <h2>
          <FontAwesomeIcon icon={faTrophy} /> Measured performance
        </h2>
        <div className="row">
          {benchmarks.map((benchmark) => (
            <div key={benchmark.label} className="col col--3">
              <div className={styles.benchmarkCard}>
                <div className={styles.benchmarkIcon} style={{ color: 'var(--ifm-color-primary)' }}>
                  <FontAwesomeIcon icon={benchmark.icon} />
                </div>
                <div className={styles.benchmarkValue}>{benchmark.value}</div>
                <div className={styles.benchmarkLabel}>{benchmark.label}</div>
              </div>
            </div>
          ))}
        </div>
        <div className={styles.chartContainer}>
          <div className={styles.performanceChart}>
            <h3>Parse time by message size (log scale)</h3>
            <div className={styles.chartBars}>
              {bars.map((bar) => (
                <div key={bar.size} className={styles.chartBar} style={{ height: height(bar.ms) }}>
                  <span>
                    {bar.size}
                    <br />
                    {bar.label}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
        <p className={styles.sectionSubtitle}>
          Criterion benches on a GitHub-hosted runner (Intel Xeon, 4 vCPUs), release profile,
          0.5.1.{' '}
          <Link to="/docs/performance">Method and full results</Link>
        </p>
      </div>
    </section>
  );
}

function CodeExamples() {
  return (
    <section className={styles.codeExamples}>
      <div className="container">
        <h2>Parse and build in three languages</h2>
        <Tabs groupId="language">
          <TabItem value="node" label="Node.js" default>
            <CodeBlock language="js">{nodeExample}</CodeBlock>
            <div className={styles.installCommand}>
              <CodeBlock language="bash">npm install ddex-parser ddex-builder</CodeBlock>
            </div>
          </TabItem>
          <TabItem value="python" label="Python">
            <CodeBlock language="python">{pythonExample}</CodeBlock>
            <div className={styles.installCommand}>
              <CodeBlock language="bash">pip install "ddex-parser[pandas]" ddex-builder</CodeBlock>
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

function Features() {
  const features = [
    {
      icon: faLayerGroup,
      title: 'One Rust core, three languages',
      desc: 'The same parser and builder in Rust, Node.js (with TypeScript types) and Python.',
      to: '/docs/installation',
    },
    {
      icon: faEraser,
      title: 'Never invents data',
      desc: 'Absent values stay empty. A build that lacks a required value fails with an error naming the field.',
      to: '/docs/build-request#errors',
    },
    {
      icon: faCheckCircle,
      title: 'XSD-valid output',
      desc: 'ERN 4.3, 4.2 and 3.8.2 builder output is validated against the official DDEX XSDs in CI.',
      to: '/docs/ern-versions',
    },
    {
      icon: faSync,
      title: 'Official samples round trip in CI',
      desc: 'Every official ERN 4.2 and 4.3 sample is parsed, rebuilt, validated and parsed again.',
      to: '/docs/ern-versions#how-ci-checks-this',
    },
    {
      icon: faShieldAlt,
      title: 'Hardened XML parsing',
      desc: 'DTDs and external entities are rejected, entity expansion, depth and size are limited, nothing is fetched.',
      to: '/docs/security',
    },
    {
      icon: faTable,
      title: 'DataFrames for Python',
      desc: 'Releases and tracks as pandas DataFrames, and ERN built back from a DataFrame.',
      to: '/docs/python-dataframes',
    },
  ];

  return (
    <section className={styles.featureComparison}>
      <div className="container">
        <h2>What you get in 0.5.1</h2>
        <div className="row">
          {features.map((feature) => (
            <div key={feature.title} className={clsx('col col--4', styles.featureTileCol)}>
              <Link to={feature.to} className={styles.featureTile}>
                <span className={styles.featureItemIcon}>
                  <FontAwesomeIcon icon={feature.icon} />
                </span>
                <strong>{feature.title}</strong>
                <p>{feature.desc}</p>
              </Link>
            </div>
          ))}
        </div>
        <p className={styles.sectionSubtitle}>
          Pre-1.0: see the <Link to="/docs/known-issues">known issues and roadmap</Link>. Validate
          messages against the XSDs and business rules with{' '}
          <Link href="https://ddex-workbench.org">DDEX Workbench</Link>.
        </p>
      </div>
    </section>
  );
}

export default function Home() {
  return (
    <Layout
      title="DDEX ERN parser and builder"
      description="Parse and build DDEX ERN XML (3.8.x, 4.2, 4.3) with one Rust core and native bindings for Node.js and Python."
    >
      <HomepageHeader />
      <main>
        <PerformanceBenchmarks />
        <CodeExamples />
        <Features />
      </main>
    </Layout>
  );
}
