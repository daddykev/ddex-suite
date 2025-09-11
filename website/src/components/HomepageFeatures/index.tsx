import React from 'react';
import clsx from 'clsx';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  icon: string;
  description: JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'High Performance',
    icon: '⚡',
    description: (
      <>
        Rust core with sub-millisecond parsing and building. Stream large files
        with bounded memory usage.
      </>
    ),
  },
  {
    title: 'Multi-Platform',
    icon: '🌐',
    description: (
      <>
        Native bindings for Node.js and Python. WASM support for browsers.
        One codebase, every platform.
      </>
    ),
  },
  {
    title: 'Deterministic',
    icon: '🎯',
    description: (
      <>
        Byte-perfect reproducible XML generation with DB-C14N/1.0 canonicalization.
        Same input, same output, every time.
      </>
    ),
  },
  {
    title: 'Industry Presets',
    icon: '🛠️',
    description: (
      <>
        Pre-configured for Spotify, Apple Music, YouTube, and Amazon.
        Battle-tested with real-world DDEX files.
      </>
    ),
  },
  {
    title: 'Perfect Round-Trip',
    icon: '🔄',
    description: (
      <>
        Parse any DDEX file, modify it, and build it back without losing
        data or structure.
      </>
    ),
  },
  {
    title: 'Developer Friendly',
    icon: '❤️',
    description: (
      <>
        TypeScript definitions, Python type hints, comprehensive docs,
        and helpful error messages.
      </>
    ),
  },
];

function Feature({title, icon, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="feature-card text--center padding--lg">
        <div className={styles.featureIcon}>{icon}</div>
        <h3>{title}</h3>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}