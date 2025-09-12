import React, { useState, useCallback } from 'react'
import { DdexBuilderProvider } from './hooks/useDdexBuilder'
import { DdexGenerator } from './components/DdexGenerator'
import { LoadingSpinner } from './components/LoadingSpinner'
import { ErrorBoundary } from './components/ErrorBoundary'
import './App.css'

interface PerformanceMetrics {
  generationTime: number;
  xmlSize: number;
  memoryUsage?: number;
}

function App() {
  const [metrics, setMetrics] = useState<PerformanceMetrics | null>(null)

  const handleMetricsUpdate = useCallback((newMetrics: PerformanceMetrics) => {
    setMetrics(newMetrics)
  }, [])

  return (
    <div className="app">
      <header className="app-header">
        <h1>🎵 DDEX Builder React Demo</h1>
        <p>Generate professional DDEX XML using WebAssembly in React</p>
      </header>
      
      <main className="app-main">
        <ErrorBoundary>
          <DdexBuilderProvider>
            {({ builder, isLoading, error }) => {
              if (isLoading) {
                return (
                  <div className="loading-container">
                    <LoadingSpinner />
                    <p>Loading WASM module...</p>
                  </div>
                )
              }
              
              if (error) {
                return (
                  <div className="error-container">
                    <h2>❌ Failed to Load DDEX Builder</h2>
                    <p>{error.message}</p>
                    <details>
                      <summary>Error Details</summary>
                      <pre>{error.stack}</pre>
                    </details>
                  </div>
                )
              }
              
              return (
                <>
                  <DdexGenerator 
                    builder={builder} 
                    onMetricsUpdate={handleMetricsUpdate}
                  />
                  
                  {metrics && (
                    <div className="performance-metrics">
                      <h3>📊 Performance Metrics</h3>
                      <div className="metrics-grid">
                        <div className="metric">
                          <label>Generation Time:</label>
                          <span>{metrics.generationTime.toFixed(2)}ms</span>
                        </div>
                        <div className="metric">
                          <label>XML Size:</label>
                          <span>{(metrics.xmlSize / 1024).toFixed(2)}KB</span>
                        </div>
                        {metrics.memoryUsage && (
                          <div className="metric">
                            <label>Memory Usage:</label>
                            <span>{metrics.memoryUsage.toFixed(2)}MB</span>
                          </div>
                        )}
                      </div>
                    </div>
                  )}
                </>
              )
            }}
          </DdexBuilderProvider>
        </ErrorBoundary>
      </main>
      
      <footer className="app-footer">
        <p>
          Powered by <a href="https://ddex-suite.web.app" target="_blank" rel="noopener noreferrer">
            DDEX Suite
          </a> • Built with React + WebAssembly
        </p>
      </footer>
    </div>
  )
}

export default App