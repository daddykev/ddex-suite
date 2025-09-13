import React from 'react';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import AnimatedLanding from '@site/src/components/AnimatedLanding';

export default function Home() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title="High-Performance DDEX Processing"
      description="Parse and build DDEX XML with perfect fidelity. Native bindings for Node.js and Python with TypeScript support."
      wrapperClassName="landing-page">
      <div style={{
        background: '#1a0033 !important',
        minHeight: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        overflowX: 'hidden',
        position: 'relative',
        zIndex: 1
      }}>
        <AnimatedLanding />
      </div>
    </Layout>
  );
}