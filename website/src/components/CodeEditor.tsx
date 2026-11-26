import React, { useEffect, useRef, useState } from 'react';
import type * as MonacoApi from 'monaco-editor';

type Monaco = typeof MonacoApi;
type EditorOptions = MonacoApi.editor.IStandaloneEditorConstructionOptions;

// Monaco is loaded on first use (browser only), from the bundled npm package.
let monacoPromise: Promise<Monaco> | undefined;
function loadMonaco(): Promise<Monaco> {
  monacoPromise ??= import('./monaco/setup').then((m) => m.monaco);
  return monacoPromise;
}

interface CodeEditorProps {
  language: string;
  value: string;
  /** Monaco theme: 'vs' (light) or 'vs-dark'. */
  theme: string;
  options?: EditorOptions;
  onChange?: (value: string) => void;
}

/** A Monaco editor. Render it in the browser only (inside BrowserOnly). */
export default function CodeEditor({ language, value, theme, options, onChange }: CodeEditorProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<MonacoApi.editor.IStandaloneCodeEditor | null>(null);
  const monacoRef = useRef<Monaco | null>(null);
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const [ready, setReady] = useState(false);
  const [failed, setFailed] = useState(false);

  // Create the editor once; later prop changes are applied by the effects below.
  useEffect(() => {
    let disposed = false;
    let editor: MonacoApi.editor.IStandaloneCodeEditor | undefined;
    loadMonaco().then(
      (monaco) => {
        if (disposed || !containerRef.current) return;
        monacoRef.current = monaco;
        editor = monaco.editor.create(containerRef.current, { ...options, value, language, theme });
        editor.onDidChangeModelContent(() => onChangeRef.current?.(editor!.getValue()));
        editorRef.current = editor;
        setReady(true);
      },
      () => {
        if (!disposed) setFailed(true);
      },
    );
    return () => {
      disposed = true;
      editor?.getModel()?.dispose();
      editor?.dispose();
      editorRef.current = null;
    };
  }, []);

  useEffect(() => {
    const editor = editorRef.current;
    if (editor && editor.getValue() !== value) editor.setValue(value);
  }, [value, ready]);

  useEffect(() => {
    const model = editorRef.current?.getModel();
    if (model && monacoRef.current) monacoRef.current.editor.setModelLanguage(model, language);
  }, [language, ready]);

  useEffect(() => {
    monacoRef.current?.editor.setTheme(theme);
  }, [theme, ready]);

  const optionsKey = JSON.stringify(options ?? {});
  useEffect(() => {
    if (options) editorRef.current?.updateOptions(options);
  }, [optionsKey, ready]);

  return (
    <div style={{ position: 'relative', height: '100%', width: '100%' }}>
      {/* Monaco owns this element's children; React renders nothing inside it. */}
      <div ref={containerRef} style={{ height: '100%', width: '100%' }} />
      {!ready && (
        <div style={{ position: 'absolute', inset: 0, padding: '1rem' }} role="status">
          {failed ? 'The editor could not be loaded.' : 'Loading editor…'}
        </div>
      )}
    </div>
  );
}
