import React, { useState, useCallback, useEffect } from 'react';
import Layout from '@theme/Layout';
import BrowserOnly from '@docusaurus/BrowserOnly';
import { useColorMode } from '@docusaurus/theme-common';
import { Allotment } from 'allotment';
import Editor from '@monaco-editor/react';
import 'allotment/dist/style.css';
import { PARSE_SAMPLES, BUILD_SAMPLES } from '../data/playgroundSamples';

// Must match MAX_BODY_BYTES in functions/index.js.
const MAX_BODY_BYTES = 5 * 1024 * 1024;
const NARROW_QUERY = '(max-width: 768px)';

type Mode = 'parser' | 'builder';
type ApiStatus = 'checking' | 'online' | 'offline';

interface Versions {
  ddexParser: string | null;
  ddexBuilder: string | null;
}

const DEFAULT_PARSE_SAMPLE = Object.keys(PARSE_SAMPLES)[0];
const DEFAULT_BUILD_SAMPLE = Object.keys(BUILD_SAMPLES)[0];

function samplesFor(mode: Mode): Record<string, string> {
  return mode === 'parser' ? PARSE_SAMPLES : BUILD_SAMPLES;
}

function formatMB(bytes: number): string {
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

// POST a JSON body to the API and return the parsed JSON response, or throw an Error whose
// message is fit to show to the user.
async function postJson(url: string, body: string): Promise<any> {
  const size = new Blob([body]).size;
  if (size > MAX_BODY_BYTES) {
    throw new Error(
      `The input is ${formatMB(size)}; the playground accepts up to 5 MB. ` +
        'Install ddex-parser or ddex-builder to process larger files locally.',
    );
  }

  const response = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body,
  });

  let data: any = null;
  try {
    data = await response.json();
  } catch {
    // Not JSON (e.g. a proxy error page); handled below.
  }

  if (!response.ok) {
    if (response.status === 413) {
      throw new Error('The input is larger than the 5 MB limit.');
    }
    const message = data && typeof data.error === 'string' ? data.error : `Request failed (HTTP ${response.status}).`;
    throw new Error(message);
  }
  if (data === null) {
    throw new Error('The API returned an unexpected response.');
  }
  return data;
}

function useIsNarrow(): boolean {
  const [narrow, setNarrow] = useState(() => window.matchMedia(NARROW_QUERY).matches);
  useEffect(() => {
    const query = window.matchMedia(NARROW_QUERY);
    const onChange = () => setNarrow(query.matches);
    query.addEventListener('change', onChange);
    return () => query.removeEventListener('change', onChange);
  }, []);
  return narrow;
}

const paneHeaderStyle: React.CSSProperties = {
  padding: '0.5rem 1rem',
  borderBottom: '1px solid var(--ifm-color-emphasis-200)',
  backgroundColor: 'var(--ifm-color-emphasis-100)',
  fontSize: '0.9rem',
  fontWeight: 'bold',
};

function PlaygroundComponent() {
  const { colorMode } = useColorMode();
  const editorTheme = colorMode === 'dark' ? 'vs-dark' : 'light';
  const isNarrow = useIsNarrow();

  const [mode, setMode] = useState<Mode>('parser');
  const [sample, setSample] = useState<string>(DEFAULT_PARSE_SAMPLE);
  const [input, setInput] = useState<string>(PARSE_SAMPLES[DEFAULT_PARSE_SAMPLE]);
  const [output, setOutput] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [apiStatus, setApiStatus] = useState<ApiStatus>('checking');
  const [versions, setVersions] = useState<Versions | null>(null);

  // "Online" only when /api/health answers with the expected JSON.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const response = await fetch('/api/health', { headers: { Accept: 'application/json' } });
        const contentType = response.headers.get('content-type') || '';
        if (!response.ok || !contentType.includes('application/json')) {
          throw new Error('unhealthy');
        }
        const data = await response.json();
        if (data?.status !== 'ok') {
          throw new Error('unhealthy');
        }
        if (!cancelled) {
          setVersions({ ddexParser: data.ddexParser ?? null, ddexBuilder: data.ddexBuilder ?? null });
          setApiStatus('online');
        }
      } catch {
        if (!cancelled) setApiStatus('offline');
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const handleProcess = useCallback(async () => {
    if (!input.trim()) {
      setError('Please provide input.');
      setOutput('');
      return;
    }

    setLoading(true);
    setError('');
    setOutput('');

    try {
      if (mode === 'parser') {
        const parsed = await postJson('/api/parse', JSON.stringify({ xml: input }));
        setOutput(JSON.stringify(parsed, null, 2));
      } else {
        const trimmed = input.trim();
        if (trimmed.startsWith('<')) {
          throw new Error('Builder mode takes a JSON build request. Switch to Parser mode to parse XML.');
        }
        let request: unknown;
        try {
          request = JSON.parse(trimmed);
        } catch {
          throw new Error('The input is not valid JSON.');
        }
        const { xml } = await postJson('/api/build', JSON.stringify(request));
        setOutput(xml);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Something went wrong.');
    } finally {
      setLoading(false);
    }
  }, [input, mode]);

  const handleModeChange = useCallback((newMode: Mode) => {
    const name = newMode === 'parser' ? DEFAULT_PARSE_SAMPLE : DEFAULT_BUILD_SAMPLE;
    setMode(newMode);
    setSample(name);
    setInput(samplesFor(newMode)[name]);
    setOutput('');
    setError('');
  }, []);

  const loadSample = useCallback(
    (name: string) => {
      const content = samplesFor(mode)[name];
      if (content === undefined) return;
      setSample(name);
      setInput(content);
      setOutput('');
      setError('');
    },
    [mode],
  );

  const exportOutput = useCallback(() => {
    if (!output) return;
    const isJson = mode === 'parser';
    const blob = new Blob([output], { type: isJson ? 'application/json' : 'application/xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = isJson ? 'parsed-data.json' : 'generated.xml';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }, [output, mode]);

  const inputLanguage = mode === 'parser' ? 'xml' : 'json';
  const outputLanguage = mode === 'parser' ? 'json' : 'xml';
  const editorOptions = {
    minimap: { enabled: false },
    lineNumbers: 'on' as const,
    wordWrap: 'on' as const,
    automaticLayout: true,
    scrollBeyondLastLine: false,
    fontSize: 14,
  };

  const inputPane = (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={paneHeaderStyle}>
        Input ({mode === 'parser' ? 'ERN XML' : 'JSON build request'})
      </div>
      <div style={{ flex: 1, minHeight: 0 }}>
        <Editor
          language={inputLanguage}
          value={input}
          onChange={(value) => setInput(value || '')}
          options={{ ...editorOptions, ariaLabel: 'Input editor' }}
          theme={editorTheme}
        />
      </div>
    </div>
  );

  const outputPane = (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={paneHeaderStyle}>
        Output ({mode === 'parser' ? 'parsed JSON' : 'ERN XML'})
      </div>
      <div style={{ flex: 1, minHeight: 0 }}>
        <Editor
          language={outputLanguage}
          value={output}
          options={{ ...editorOptions, readOnly: true, ariaLabel: 'Output (read only)' }}
          theme={editorTheme}
        />
      </div>
    </div>
  );

  const statusText =
    apiStatus === 'online' ? 'API online' : apiStatus === 'offline' ? 'API offline' : 'Checking API…';
  const statusColor =
    apiStatus === 'online'
      ? 'var(--ifm-color-success)'
      : apiStatus === 'offline'
        ? 'var(--ifm-color-danger)'
        : 'var(--ifm-color-warning)';

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: isNarrow ? 'auto' : 'calc(100vh - var(--ifm-navbar-height))',
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: '1rem',
          borderBottom: '1px solid var(--ifm-color-emphasis-200)',
          backgroundColor: 'var(--ifm-background-color)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '1rem', flexWrap: 'wrap' }}>
          <h1 style={{ margin: 0, fontSize: '1.5rem' }}>Playground</h1>
          <div style={{ display: 'flex', gap: '0.5rem' }} role="group" aria-label="Mode">
            <button
              type="button"
              className={`button ${mode === 'parser' ? 'button--primary' : 'button--secondary'}`}
              aria-pressed={mode === 'parser'}
              onClick={() => handleModeChange('parser')}
            >
              Parser
            </button>
            <button
              type="button"
              className={`button ${mode === 'builder' ? 'button--primary' : 'button--secondary'}`}
              aria-pressed={mode === 'builder'}
              onClick={() => handleModeChange('builder')}
            >
              Builder
            </button>
          </div>
          <div
            role="status"
            aria-live="polite"
            style={{ marginLeft: 'auto', fontSize: '0.9rem', fontWeight: 500 }}
          >
            <span aria-hidden="true" style={{ color: statusColor }}>
              ●{' '}
            </span>
            {statusText}
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', flexWrap: 'wrap' }}>
          <div>
            <label htmlFor="playground-sample" style={{ marginRight: '0.5rem', fontSize: '0.9rem' }}>
              Sample:
            </label>
            <select id="playground-sample" value={sample} onChange={(e) => loadSample(e.target.value)}>
              {Object.keys(samplesFor(mode)).map((name) => (
                <option key={name} value={name}>
                  {name}
                </option>
              ))}
            </select>
          </div>

          <button
            type="button"
            className="button button--primary"
            onClick={handleProcess}
            disabled={loading || apiStatus !== 'online'}
          >
            {loading
              ? 'Processing…'
              : apiStatus === 'checking'
                ? 'Checking API…'
                : apiStatus === 'offline'
                  ? 'API offline'
                  : mode === 'parser'
                    ? 'Parse XML'
                    : 'Build XML'}
          </button>

          {output && (
            <button type="button" className="button button--secondary" onClick={exportOutput}>
              Export {mode === 'parser' ? 'JSON' : 'XML'}
            </button>
          )}
        </div>

        <div role="alert" aria-live="assertive">
          {error && (
            <div
              style={{
                marginTop: '1rem',
                padding: '0.5rem',
                backgroundColor: 'var(--ifm-color-danger-contrast-background)',
                color: 'var(--ifm-color-danger-contrast-foreground)',
                borderRadius: '4px',
                fontSize: '0.9rem',
              }}
            >
              {error}
            </div>
          )}
        </div>
      </div>

      {/* Editors: side by side, or stacked on narrow screens */}
      {isNarrow ? (
        <div style={{ display: 'flex', flexDirection: 'column' }}>
          <div style={{ height: '50vh', minHeight: '320px' }}>{inputPane}</div>
          <div style={{ height: '50vh', minHeight: '320px' }}>{outputPane}</div>
        </div>
      ) : (
        <div style={{ flex: 1, minHeight: 0 }}>
          <Allotment defaultSizes={[50, 50]}>
            <Allotment.Pane>{inputPane}</Allotment.Pane>
            <Allotment.Pane>{outputPane}</Allotment.Pane>
          </Allotment>
        </div>
      )}

      {/* Footer */}
      <div
        style={{
          padding: '0.5rem 1rem',
          borderTop: '1px solid var(--ifm-color-emphasis-200)',
          backgroundColor: 'var(--ifm-background-color)',
          fontSize: '0.8rem',
          color: 'var(--ifm-color-emphasis-700)',
        }}
      >
        {versions ? (
          <>
            Running ddex-parser {versions.ddexParser ?? 'unknown'} and ddex-builder{' '}
            {versions.ddexBuilder ?? 'unknown'} on Firebase Cloud Functions.
          </>
        ) : (
          <>Runs ddex-parser and ddex-builder on Firebase Cloud Functions.</>
        )}{' '}
        Requests are limited to 5 MB. The build request format is the ddex-builder{' '}
        <code>BuildRequest</code> type (snake_case fields; <code>header</code> and{' '}
        <code>version</code> are required).
      </div>
    </div>
  );
}

export default function Playground() {
  return (
    <Layout
      title="Playground"
      description="Try ddex-parser and ddex-builder in the browser: parse DDEX ERN XML to JSON, or build ERN XML from a build request."
    >
      <BrowserOnly fallback={<div style={{ padding: '1rem' }}>Loading playground…</div>}>
        {() => <PlaygroundComponent />}
      </BrowserOnly>
    </Layout>
  );
}
