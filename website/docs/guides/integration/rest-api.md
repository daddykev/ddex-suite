# REST API Integration

Build REST APIs around DDEX Suite to provide web-based DDEX processing capabilities.

## Overview

REST API integration enables:
- Web-based DDEX parsing and building
- Microservice architectures
- Third-party integrations
- Scalable processing endpoints
- Real-time data transformation

## Express.js API Server

### Basic Setup

```typescript
import express from 'express';
import multer from 'multer';
import { DDEXParser } from 'ddex-parser';
import { DDEXBuilder } from 'ddex-builder';

const app = express();
const upload = multer({ storage: multer.memoryStorage() });
const parser = new DDEXParser();
const builder = new DDEXBuilder();

app.use(express.json({ limit: '50mb' }));

// Parse DDEX XML endpoint
app.post('/api/ddex/parse', upload.single('ddex'), async (req, res) => {
  try {
    const xmlContent = req.file?.buffer.toString() || req.body.xml;
    
    if (!xmlContent) {
      return res.status(400).json({ error: 'No DDEX XML provided' });
    }

    const result = await parser.parse(xmlContent);
    
    res.json({
      success: true,
      data: {
        version: result.version,
        flat: result.flat,
        graph: result.graph,
        metadata: {
          processingTimeMs: result.processingTimeMs,
          fileSize: xmlContent.length
        }
      }
    });
    
  } catch (error) {
    res.status(400).json({
      success: false,
      error: error.message,
      details: error.details || null
    });
  }
});

// Build DDEX XML endpoint
app.post('/api/ddex/build', async (req, res) => {
  try {
    const { data, options = {} } = req.body;
    
    if (!data) {
      return res.status(400).json({ error: 'No build data provided' });
    }

    // Validate before building
    const validation = await builder.validate(data, {
      level: options.validationLevel || 'standard'
    });

    if (!validation.isValid) {
      return res.status(400).json({
        success: false,
        error: 'Validation failed',
        validationErrors: validation.errors,
        warnings: validation.warnings
      });
    }

    const xml = await builder.build(data, options);
    
    res.json({
      success: true,
      data: {
        xml,
        validation: {
          warnings: validation.warnings,
          quality: validation.qualityScore
        }
      }
    });
    
  } catch (error) {
    res.status(400).json({
      success: false,
      error: error.message
    });
  }
});

// Batch processing endpoint
app.post('/api/ddex/batch', async (req, res) => {
  try {
    const { operation, items, options = {} } = req.body;
    
    let results;
    if (operation === 'parse') {
      results = await parser.parseBatch(items, options);
    } else if (operation === 'build') {
      results = await builder.buildBatch(items, options);
    } else {
      return res.status(400).json({ error: 'Invalid operation' });
    }
    
    res.json({
      success: true,
      data: {
        total: items.length,
        successful: results.success.length,
        failed: results.errors.length,
        results: results.success,
        errors: results.errors
      }
    });
    
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

// Health check
app.get('/api/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    version: process.env.npm_package_version
  });
});

app.listen(3000, () => {
  console.log('DDEX Suite API server running on port 3000');
});
```

## FastAPI Python Server

```python
from fastapi import FastAPI, UploadFile, File, HTTPException
from fastapi.responses import JSONResponse
from pydantic import BaseModel
from typing import List, Optional, Dict, Any
import asyncio
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

app = FastAPI(title="DDEX Suite API", version="1.0.0")

# Initialize processors
parser = DDEXParser()
builder = DDEXBuilder()

# Request models
class BuildRequest(BaseModel):
    data: Dict[str, Any]
    options: Optional[Dict[str, Any]] = {}

class BatchRequest(BaseModel):
    operation: str  # 'parse' or 'build'
    items: List[Dict[str, Any]]
    options: Optional[Dict[str, Any]] = {}

@app.post("/api/ddex/parse")
async def parse_ddex(file: UploadFile = File(...)):
    """Parse DDEX XML file"""
    try:
        content = await file.read()
        xml_content = content.decode('utf-8')
        
        result = parser.parse(xml_content)
        
        return {
            "success": True,
            "data": {
                "version": result.version,
                "flat": result.flat.to_dict(),
                "graph": result.graph.to_dict(),
                "metadata": {
                    "processing_time_ms": result.processing_time_ms,
                    "file_size": len(xml_content),
                    "filename": file.filename
                }
            }
        }
        
    except Exception as e:
        raise HTTPException(status_code=400, detail={
            "success": False,
            "error": str(e),
            "type": type(e).__name__
        })

@app.post("/api/ddex/parse/text")
async def parse_ddex_text(xml: str):
    """Parse DDEX XML from text"""
    try:
        result = parser.parse(xml)
        
        return {
            "success": True,
            "data": {
                "version": result.version,
                "flat": result.flat.to_dict(),
                "graph": result.graph.to_dict()
            }
        }
        
    except Exception as e:
        raise HTTPException(status_code=400, detail={
            "success": False,
            "error": str(e)
        })

@app.post("/api/ddex/build")
async def build_ddex(request: BuildRequest):
    """Build DDEX XML from data"""
    try:
        # Validate first
        validation = builder.validate(
            request.data, 
            level=request.options.get('validation_level', 'standard')
        )
        
        if not validation.is_valid:
            raise HTTPException(status_code=400, detail={
                "success": False,
                "error": "Validation failed",
                "validation_errors": validation.errors,
                "warnings": validation.warnings
            })
        
        xml = builder.build(request.data, **request.options)
        
        return {
            "success": True,
            "data": {
                "xml": xml,
                "validation": {
                    "warnings": validation.warnings,
                    "quality_score": validation.quality_score
                }
            }
        }
        
    except Exception as e:
        raise HTTPException(status_code=400, detail={
            "success": False,
            "error": str(e)
        })

@app.post("/api/ddex/batch")
async def batch_process(request: BatchRequest):
    """Batch process multiple DDEX operations"""
    try:
        if request.operation == "parse":
            results = parser.parse_batch(request.items, **request.options)
        elif request.operation == "build":
            results = builder.build_batch(request.items, **request.options)
        else:
            raise HTTPException(status_code=400, detail="Invalid operation")
        
        return {
            "success": True,
            "data": {
                "total": len(request.items),
                "successful": len(results.success),
                "failed": len(results.errors),
                "results": results.success,
                "errors": results.errors
            }
        }
        
    except Exception as e:
        raise HTTPException(status_code=500, detail={
            "success": False,
            "error": str(e)
        })

@app.get("/api/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "timestamp": datetime.utcnow().isoformat(),
        "services": {
            "parser": "operational",
            "builder": "operational"
        }
    }

# Run with: uvicorn main:app --reload --port 8000
```

## Actix Web Rust Server

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Result, middleware::Logger};
use actix_multipart::Multipart;
use serde::{Deserialize, Serialize};
use ddex_parser::DDEXParser;
use ddex_builder::DDEXBuilder;

#[derive(Deserialize)]
struct BuildRequest {
    data: serde_json::Value,
    options: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

async fn parse_ddex(mut payload: Multipart) -> Result<HttpResponse> {
    let parser = DDEXParser::new();
    
    while let Some(item) = payload.try_next().await? {
        let field = item?;
        let content_disposition = field.content_disposition();
        
        if let Some("ddex") = content_disposition.get_name() {
            let bytes = field.bytes().await?;
            let xml_content = String::from_utf8(bytes.to_vec())
                .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UTF-8"))?;
                
            match parser.parse(&xml_content) {
                Ok(result) => {
                    let response = ApiResponse::success(serde_json::json!({
                        "version": result.version(),
                        "flat": result.flat(),
                        "graph": result.graph(),
                        "metadata": {
                            "processing_time_ms": result.processing_time_ms(),
                            "file_size": xml_content.len()
                        }
                    }));
                    
                    return Ok(HttpResponse::Ok().json(response));
                }
                Err(e) => {
                    let response = ApiResponse::<()>::error(e.to_string());
                    return Ok(HttpResponse::BadRequest().json(response));
                }
            }
        }
    }
    
    let response = ApiResponse::<()>::error("No DDEX file found".to_string());
    Ok(HttpResponse::BadRequest().json(response))
}

async fn build_ddex(request: web::Json<BuildRequest>) -> Result<HttpResponse> {
    let builder = DDEXBuilder::new();
    
    // Validate first
    match builder.validate(&request.data) {
        Ok(validation) => {
            if !validation.is_valid() {
                let response = ApiResponse::<()>::error(format!(
                    "Validation failed: {:?}", 
                    validation.errors()
                ));
                return Ok(HttpResponse::BadRequest().json(response));
            }
        }
        Err(e) => {
            let response = ApiResponse::<()>::error(e.to_string());
            return Ok(HttpResponse::BadRequest().json(response));
        }
    }
    
    // Build XML
    match builder.build(&request.data) {
        Ok(xml) => {
            let response = ApiResponse::success(serde_json::json!({
                "xml": xml
            }));
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            let response = ApiResponse::<()>::error(e.to_string());
            Ok(HttpResponse::BadRequest().json(response))
        }
    }
}

async fn health_check() -> Result<HttpResponse> {
    let response = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    });
    
    Ok(HttpResponse::Ok().json(response))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/api/ddex/parse", web::post().to(parse_ddex))
            .route("/api/ddex/build", web::post().to(build_ddex))
            .route("/api/health", web::get().to(health_check))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Client Libraries

### JavaScript/TypeScript Client

```typescript
export class DDEXClient {
  constructor(private baseUrl: string, private apiKey?: string) {}

  private async request(endpoint: string, options: RequestInit = {}): Promise<any> {
    const url = `${this.baseUrl}${endpoint}`;
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    if (this.apiKey) {
      headers['Authorization'] = `Bearer ${this.apiKey}`;
    }

    const response = await fetch(url, {
      ...options,
      headers,
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(data.error || `HTTP ${response.status}`);
    }

    return data;
  }

  async parse(xmlContent: string): Promise<ParseResult> {
    return this.request('/api/ddex/parse/text', {
      method: 'POST',
      body: JSON.stringify({ xml: xmlContent }),
    });
  }

  async parseFile(file: File): Promise<ParseResult> {
    const formData = new FormData();
    formData.append('ddex', file);

    return this.request('/api/ddex/parse', {
      method: 'POST',
      body: formData,
      headers: {}, // Remove Content-Type to let browser set it
    });
  }

  async build(data: any, options: BuildOptions = {}): Promise<BuildResult> {
    return this.request('/api/ddex/build', {
      method: 'POST',
      body: JSON.stringify({ data, options }),
    });
  }

  async batchProcess(operation: 'parse' | 'build', items: any[], options: BatchOptions = {}): Promise<BatchResult> {
    return this.request('/api/ddex/batch', {
      method: 'POST',
      body: JSON.stringify({ operation, items, options }),
    });
  }

  async healthCheck(): Promise<HealthStatus> {
    return this.request('/api/health');
  }
}

// Usage
const client = new DDEXClient('https://api.example.com', 'your-api-key');

try {
  const parseResult = await client.parse(xmlContent);
  console.log('Parsed successfully:', parseResult.data);
} catch (error) {
  console.error('Parse failed:', error.message);
}
```

### Python Client

```python
import requests
from typing import Dict, List, Optional, Any
import json

class DDEXClient:
    def __init__(self, base_url: str, api_key: Optional[str] = None):
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
        
        if api_key:
            self.session.headers.update({
                'Authorization': f'Bearer {api_key}'
            })
    
    def _request(self, method: str, endpoint: str, **kwargs) -> Dict[str, Any]:
        url = f"{self.base_url}{endpoint}"
        response = self.session.request(method, url, **kwargs)
        
        try:
            data = response.json()
        except json.JSONDecodeError:
            data = {"error": "Invalid JSON response"}
        
        if not response.ok:
            raise Exception(data.get('error', f'HTTP {response.status_code}'))
        
        return data
    
    def parse(self, xml_content: str) -> Dict[str, Any]:
        """Parse DDEX XML content"""
        return self._request('POST', '/api/ddex/parse/text', 
                           json={'xml': xml_content})
    
    def parse_file(self, file_path: str) -> Dict[str, Any]:
        """Parse DDEX XML file"""
        with open(file_path, 'rb') as f:
            files = {'ddex': f}
            return self._request('POST', '/api/ddex/parse', files=files)
    
    def build(self, data: Dict[str, Any], options: Dict[str, Any] = None) -> Dict[str, Any]:
        """Build DDEX XML from data"""
        payload = {'data': data}
        if options:
            payload['options'] = options
        
        return self._request('POST', '/api/ddex/build', json=payload)
    
    def batch_process(self, operation: str, items: List[Dict[str, Any]], 
                     options: Dict[str, Any] = None) -> Dict[str, Any]:
        """Batch process multiple items"""
        payload = {
            'operation': operation,
            'items': items
        }
        if options:
            payload['options'] = options
        
        return self._request('POST', '/api/ddex/batch', json=payload)
    
    def health_check(self) -> Dict[str, Any]:
        """Check API health"""
        return self._request('GET', '/api/health')

# Usage
client = DDEXClient('https://api.example.com', 'your-api-key')

try:
    with open('release.xml', 'r') as f:
        result = client.parse(f.read())
        print(f"Parsed successfully: {result['data']['version']}")
except Exception as e:
    print(f"Parse failed: {e}")
```

## API Documentation

### OpenAPI/Swagger Specification

```yaml
openapi: 3.0.0
info:
  title: DDEX Suite API
  version: 1.0.0
  description: REST API for DDEX XML processing
  
paths:
  /api/ddex/parse:
    post:
      summary: Parse DDEX XML file
      requestBody:
        content:
          multipart/form-data:
            schema:
              type: object
              properties:
                ddex:
                  type: string
                  format: binary
      responses:
        200:
          description: Successfully parsed
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ParseResponse'
  
  /api/ddex/build:
    post:
      summary: Build DDEX XML from data
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/BuildRequest'
      responses:
        200:
          description: Successfully built
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/BuildResponse'

components:
  schemas:
    ParseResponse:
      type: object
      properties:
        success:
          type: boolean
        data:
          type: object
          properties:
            version:
              type: string
            flat:
              type: object
            graph:
              type: object
    
    BuildRequest:
      type: object
      properties:
        data:
          type: object
        options:
          type: object
    
    BuildResponse:
      type: object
      properties:
        success:
          type: boolean
        data:
          type: object
          properties:
            xml:
              type: string
```

## Deployment Considerations

### Docker

```dockerfile
FROM node:18-alpine

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .

EXPOSE 3000
CMD ["npm", "start"]
```

### Environment Configuration

```typescript
const config = {
  port: process.env.PORT || 3000,
  maxFileSize: process.env.MAX_FILE_SIZE || '50mb',
  apiKey: process.env.API_KEY,
  corsOrigins: process.env.CORS_ORIGINS?.split(',') || ['*'],
  rateLimit: {
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 100 // limit each IP to 100 requests per windowMs
  }
};
```

### Security

```typescript
import rateLimit from 'express-rate-limit';
import helmet from 'helmet';
import cors from 'cors';

app.use(helmet());
app.use(cors({ origin: config.corsOrigins }));
app.use(rateLimit(config.rateLimit));

// API key middleware
app.use('/api', (req, res, next) => {
  const apiKey = req.headers.authorization?.replace('Bearer ', '');
  
  if (!apiKey || apiKey !== config.apiKey) {
    return res.status(401).json({ error: 'Invalid API key' });
  }
  
  next();
});
```

## Best Practices

1. **Implement proper error handling** with consistent error formats
2. **Use request validation** to sanitize input data
3. **Add rate limiting** to prevent abuse
4. **Implement authentication** for production APIs
5. **Log requests and responses** for debugging
6. **Use async processing** for large files
7. **Implement health checks** for monitoring
8. **Version your API** for backward compatibility
9. **Document thoroughly** with OpenAPI/Swagger
10. **Add proper CORS headers** for web clients