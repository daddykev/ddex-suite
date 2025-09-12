import React, { useState, useCallback } from 'react'
import { DdexBuilder } from 'ddex-builder-wasm'

interface PerformanceMetrics {
  generationTime: number
  xmlSize: number
  memoryUsage?: number
}

interface DdexGeneratorProps {
  builder: DdexBuilder | null
  onMetricsUpdate: (metrics: PerformanceMetrics) => void
}

interface FormData {
  releaseId: string
  title: string
  artist: string
  releaseType: string
  label: string
  releaseDate: string
  genre: string
  upc: string
  metadata: string
}

const initialFormData: FormData = {
  releaseId: '',
  title: '',
  artist: '',
  releaseType: 'Album',
  label: '',
  releaseDate: new Date().toISOString().split('T')[0],
  genre: '',
  upc: '',
  metadata: ''
}

const exampleData = {
  single: {
    releaseId: 'REL2024001',
    title: 'Summer Nights',
    artist: 'Luna Rodriguez',
    releaseType: 'Single',
    label: 'Midnight Records',
    releaseDate: '2024-06-15',
    genre: 'Pop',
    upc: '123456789012'
  },
  album: {
    releaseId: 'ALB2024001',
    title: 'Digital Dreams',
    artist: 'The Synthetic Collective',
    releaseType: 'Album',
    label: 'Future Sound Records',
    releaseDate: '2024-09-01',
    genre: 'Electronic, Synthwave',
    upc: '987654321098',
    metadata: '{"copyright": "2024 The Synthetic Collective", "producer": "Alex Dreamweaver", "total_tracks": 12}'
  }
}

export function DdexGenerator({ builder, onMetricsUpdate }: DdexGeneratorProps) {
  const [formData, setFormData] = useState<FormData>(initialFormData)
  const [generatedXml, setGeneratedXml] = useState<string>('')
  const [isGenerating, setIsGenerating] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleInputChange = useCallback((field: keyof FormData, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }))
  }, [])

  const loadExample = useCallback((type: keyof typeof exampleData) => {
    const example = exampleData[type]
    setFormData({ ...initialFormData, ...example })
    setError(null)
  }, [])

  const clearForm = useCallback(() => {
    setFormData(initialFormData)
    setGeneratedXml('')
    setError(null)
  }, [])

  const generateXml = useCallback(async () => {
    if (!builder) {
      setError('DDEX Builder not available')
      return
    }

    if (!formData.releaseId.trim() || !formData.title.trim() || !formData.artist.trim()) {
      setError('Please fill in required fields: Release ID, Title, and Artist')
      return
    }

    setIsGenerating(true)
    setError(null)

    try {
      const startTime = performance.now()

      // Parse metadata if provided
      let metadata = {}
      if (formData.metadata.trim()) {
        try {
          metadata = JSON.parse(formData.metadata)
        } catch (e) {
          throw new Error(`Invalid JSON in metadata: ${e instanceof Error ? e.message : 'Unknown error'}`)
        }
      }

      // Build release data
      const releaseData = {
        release_id: formData.releaseId,
        title: formData.title,
        artist: formData.artist,
        release_type: formData.releaseType,
        label: formData.label || undefined,
        release_date: formData.releaseDate || undefined,
        genre: formData.genre || undefined,
        upc: formData.upc || undefined,
        ...metadata
      }

      // Generate XML using WASM builder
      const xml = builder.build_release_simple(JSON.stringify(releaseData))
      
      const endTime = performance.now()
      const generationTime = endTime - startTime

      setGeneratedXml(xml)

      // Update performance metrics
      const metrics: PerformanceMetrics = {
        generationTime,
        xmlSize: xml.length,
        memoryUsage: (performance as any).memory?.usedJSHeapSize / 1024 / 1024
      }
      
      onMetricsUpdate(metrics)

    } catch (err) {
      console.error('Failed to generate DDEX XML:', err)
      setError(err instanceof Error ? err.message : 'Unknown error occurred')
    } finally {
      setIsGenerating(false)
    }
  }, [builder, formData, onMetricsUpdate])

  const downloadXml = useCallback(() => {
    if (!generatedXml) return

    const blob = new Blob([generatedXml], { type: 'application/xml' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    
    a.href = url
    a.download = `ddex-${formData.releaseId || Date.now()}.xml`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }, [generatedXml, formData.releaseId])

  const copyToClipboard = useCallback(async () => {
    if (!generatedXml) return

    try {
      await navigator.clipboard.writeText(generatedXml)
      alert('XML copied to clipboard!')
    } catch (err) {
      console.error('Failed to copy to clipboard:', err)
      // Fallback for older browsers
      const textArea = document.createElement('textarea')
      textArea.value = generatedXml
      document.body.appendChild(textArea)
      textArea.select()
      document.execCommand('copy')
      document.body.removeChild(textArea)
      alert('XML copied to clipboard!')
    }
  }, [generatedXml])

  return (
    <div className="ddex-generator">
      <div className="form-section">
        <div className="example-buttons">
          <button 
            type="button" 
            onClick={() => loadExample('single')}
            className="example-btn"
          >
            Load Single Example
          </button>
          <button 
            type="button" 
            onClick={() => loadExample('album')}
            className="example-btn"
          >
            Load Album Example
          </button>
          <button 
            type="button" 
            onClick={clearForm}
            className="example-btn clear"
          >
            Clear Form
          </button>
        </div>

        <form onSubmit={(e) => { e.preventDefault(); generateXml(); }}>
          <div className="form-grid">
            <div className="form-group">
              <label htmlFor="releaseId">Release ID *</label>
              <input
                id="releaseId"
                type="text"
                value={formData.releaseId}
                onChange={(e) => handleInputChange('releaseId', e.target.value)}
                placeholder="e.g., REL2024001"
                required
              />
            </div>

            <div className="form-group">
              <label htmlFor="releaseType">Release Type</label>
              <select
                id="releaseType"
                value={formData.releaseType}
                onChange={(e) => handleInputChange('releaseType', e.target.value)}
              >
                <option value="Album">Album</option>
                <option value="Single">Single</option>
                <option value="EP">EP</option>
                <option value="Compilation">Compilation</option>
              </select>
            </div>

            <div className="form-group">
              <label htmlFor="title">Title *</label>
              <input
                id="title"
                type="text"
                value={formData.title}
                onChange={(e) => handleInputChange('title', e.target.value)}
                placeholder="e.g., My Amazing Album"
                required
              />
            </div>

            <div className="form-group">
              <label htmlFor="artist">Artist *</label>
              <input
                id="artist"
                type="text"
                value={formData.artist}
                onChange={(e) => handleInputChange('artist', e.target.value)}
                placeholder="e.g., John Doe"
                required
              />
            </div>

            <div className="form-group">
              <label htmlFor="label">Record Label</label>
              <input
                id="label"
                type="text"
                value={formData.label}
                onChange={(e) => handleInputChange('label', e.target.value)}
                placeholder="e.g., Awesome Records"
              />
            </div>

            <div className="form-group">
              <label htmlFor="releaseDate">Release Date</label>
              <input
                id="releaseDate"
                type="date"
                value={formData.releaseDate}
                onChange={(e) => handleInputChange('releaseDate', e.target.value)}
              />
            </div>

            <div className="form-group">
              <label htmlFor="genre">Genre</label>
              <input
                id="genre"
                type="text"
                value={formData.genre}
                onChange={(e) => handleInputChange('genre', e.target.value)}
                placeholder="e.g., Pop, Rock, Jazz"
              />
            </div>

            <div className="form-group">
              <label htmlFor="upc">UPC/EAN</label>
              <input
                id="upc"
                type="text"
                value={formData.upc}
                onChange={(e) => handleInputChange('upc', e.target.value)}
                placeholder="e.g., 123456789012"
              />
            </div>
          </div>

          <div className="form-group full-width">
            <label htmlFor="metadata">Additional Metadata (JSON)</label>
            <textarea
              id="metadata"
              value={formData.metadata}
              onChange={(e) => handleInputChange('metadata', e.target.value)}
              placeholder='{"copyright": "2024 Artist Name", "producer": "Producer Name"}'
              rows={3}
            />
          </div>

          {error && (
            <div className="error-message">
              <strong>Error:</strong> {error}
            </div>
          )}

          <button 
            type="submit" 
            disabled={isGenerating}
            className="generate-btn"
          >
            {isGenerating ? '🔄 Generating...' : '🚀 Generate DDEX XML'}
          </button>
        </form>
      </div>

      {generatedXml && (
        <div className="xml-output">
          <h3>Generated DDEX XML</h3>
          
          <div className="xml-actions">
            <button onClick={downloadXml} className="action-btn">
              📥 Download
            </button>
            <button onClick={copyToClipboard} className="action-btn">
              📋 Copy
            </button>
          </div>

          <div className="xml-content">
            <pre><code>{generatedXml}</code></pre>
          </div>
        </div>
      )}
    </div>
  )
}