import React, { useState } from 'react';
import Layout from '@theme/Layout';
import CodeBlock from '@theme/CodeBlock';
// TODO: Use WASM bindings instead of Node.js bindings
// import { DdexParser } from 'ddex-parser';
// import { DdexBuilder } from 'ddex-builder';

export default function Playground() {
  const [input, setInput] = useState('');
  const [output, setOutput] = useState('');
  const [mode, setMode] = useState<'parse' | 'build'>('parse');
  
  const handleProcess = async () => {
    // TODO: Implement with WASM bindings
    setOutput('Playground functionality coming soon! WASM bindings are being integrated.');
  };

  return (
    <Layout title="Playground" description="Try DDEX Suite in your browser">
      <div className="container margin-vert--lg">
        <h1>DDEX Suite Playground</h1>
        <div className="row">
          <div className="col col--6">
            <h3>Input</h3>
            <select onChange={(e) => setMode(e.target.value as 'parse' | 'build')}>
              <option value="parse">Parse XML → JSON</option>
              <option value="build">Build JSON → XML</option>
            </select>
            <textarea
              value={input}
              onChange={(e) => setInput(e.target.value)}
              style={{ width: '100%', height: '400px', fontFamily: 'monospace' }}
              placeholder={mode === 'parse' ? 'Paste DDEX XML here...' : 'Paste JSON here...'}
            />
            <button onClick={handleProcess} className="button button--primary">
              {mode === 'parse' ? 'Parse' : 'Build'}
            </button>
          </div>
          <div className="col col--6">
            <h3>Output</h3>
            <CodeBlock language={mode === 'parse' ? 'json' : 'xml'}>
              {output || '// Output will appear here'}
            </CodeBlock>
          </div>
        </div>
      </div>
    </Layout>
  );
}