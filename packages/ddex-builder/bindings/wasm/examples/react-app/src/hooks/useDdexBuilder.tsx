import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react'
import init, { DdexBuilder } from 'ddex-builder-wasm'

interface DdexBuilderContextType {
  builder: DdexBuilder | null
  isLoading: boolean
  error: Error | null
}

const DdexBuilderContext = createContext<DdexBuilderContextType>({
  builder: null,
  isLoading: true,
  error: null
})

interface DdexBuilderProviderProps {
  children: (context: DdexBuilderContextType) => ReactNode
}

export function DdexBuilderProvider({ children }: DdexBuilderProviderProps) {
  const [builder, setBuilder] = useState<DdexBuilder | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  useEffect(() => {
    let isMounted = true

    async function initializeWasm() {
      try {
        console.log('Initializing DDEX Builder WASM...')
        
        // Initialize the WASM module
        await init()
        
        if (isMounted) {
          const builderInstance = new DdexBuilder()
          setBuilder(builderInstance)
          setIsLoading(false)
          console.log('DDEX Builder WASM initialized successfully')
        }
      } catch (err) {
        console.error('Failed to initialize DDEX Builder WASM:', err)
        
        if (isMounted) {
          setError(err instanceof Error ? err : new Error(String(err)))
          setIsLoading(false)
        }
      }
    }

    initializeWasm()

    return () => {
      isMounted = false
    }
  }, [])

  const contextValue: DdexBuilderContextType = {
    builder,
    isLoading,
    error
  }

  return children(contextValue)
}

export function useDdexBuilder(): DdexBuilderContextType {
  const context = useContext(DdexBuilderContext)
  
  if (!context) {
    throw new Error('useDdexBuilder must be used within a DdexBuilderProvider')
  }
  
  return context
}