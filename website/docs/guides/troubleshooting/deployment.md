# Deployment Issues

Common deployment problems and solutions for DDEX Suite.

## Common Deployment Issues

### Package Installation Problems

**Issue**: `npm install ddex-parser` fails

**Solutions**:
```bash
# Clear npm cache
npm cache clean --force

# Use specific registry
npm install ddex-parser --registry https://registry.npmjs.org/

# Install with legacy peer deps
npm install --legacy-peer-deps
```

**Issue**: Python package installation fails

**Solutions**:
```bash
# Update pip
pip install --upgrade pip

# Use specific index
pip install ddex-parser -i https://pypi.org/simple/

# Force reinstall
pip install --force-reinstall ddex-parser
```

### Version Compatibility Issues

**Issue**: Conflicting dependency versions

**Solutions**:
```json
// package.json - specify exact versions
{
  "dependencies": {
    "ddex-parser": "0.2.5",
    "ddex-builder": "0.2.5"
  },
  "overrides": {
    "some-conflicting-package": "1.0.0"
  }
}
```

```python
# requirements.txt - pin versions
ddex-parser==0.2.5
ddex-builder==0.2.5
```

### Docker Deployment Issues

**Issue**: Container fails to start

```dockerfile
# Dockerfile optimization
FROM node:18-alpine

# Install system dependencies
RUN apk add --no-cache python3 make g++

WORKDIR /app

# Copy package files first (better caching)
COPY package*.json ./
RUN npm ci --only=production

COPY . .

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD node healthcheck.js

CMD ["npm", "start"]
```

## Environment-Specific Issues

### Development Environment

**Common Issues**:
- Missing development dependencies
- Environment variable configuration
- Database connection issues

**Solutions**:
```bash
# Development setup script
#!/bin/bash
set -e

echo "Setting up DDEX Suite development environment..."

# Install dependencies
npm install

# Setup environment variables
cp .env.example .env

# Initialize database
npm run db:migrate

# Verify installation
npm run test

echo "Development environment ready!"
```

### Production Deployment

**Health Checks**:
```typescript
// health-check.ts
import { DDEXParser, DDEXBuilder } from 'ddex-suite';

export async function healthCheck(): Promise<{
  status: 'healthy' | 'unhealthy';
  checks: Record<string, boolean>;
  timestamp: string;
}> {
  const checks = {
    parser: false,
    builder: false,
    memory: false,
    dependencies: false
  };

  try {
    // Test parser
    const parser = new DDEXParser();
    const testXml = '<test>minimal</test>';
    await parser.parse(testXml);
    checks.parser = true;

    // Test builder
    const builder = new DDEXBuilder();
    const testData = { releases: [] };
    await builder.validate(testData);
    checks.builder = true;

    // Check memory usage
    const memUsage = process.memoryUsage();
    checks.memory = memUsage.heapUsed < 1000 * 1024 * 1024; // < 1GB

    // Check dependencies
    checks.dependencies = true;

  } catch (error) {
    console.error('Health check failed:', error);
  }

  const allHealthy = Object.values(checks).every(check => check);

  return {
    status: allHealthy ? 'healthy' : 'unhealthy',
    checks,
    timestamp: new Date().toISOString()
  };
}
```

### Kubernetes Deployment

```yaml
# k8s-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ddex-processor
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ddex-processor
  template:
    metadata:
      labels:
        app: ddex-processor
    spec:
      containers:
      - name: ddex-processor
        image: ddex-processor:latest
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        env:
        - name: NODE_ENV
          value: "production"
        - name: MAX_MEMORY
          value: "1536"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: ddex-processor-service
spec:
  selector:
    app: ddex-processor
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
```

## Performance Issues in Production

### Memory Leaks

**Detection**:
```typescript
// memory-leak-detector.ts
export class MemoryLeakDetector {
  private samples: number[] = [];
  private intervalId: NodeJS.Timeout | null = null;

  startMonitoring(intervalMs = 60000) {
    this.intervalId = setInterval(() => {
      const usage = process.memoryUsage();
      this.samples.push(usage.heapUsed);
      
      // Keep only recent samples
      if (this.samples.length > 10) {
        this.samples.shift();
      }
      
      // Check for continuous growth
      if (this.samples.length >= 5) {
        const growth = this.calculateGrowthRate();
        if (growth > 0.1) { // 10% growth per minute
          console.warn(`Memory leak detected: ${growth * 100:.1f}% growth rate`);
          this.alertMemoryLeak(growth);
        }
      }
    }, intervalMs);
  }

  private calculateGrowthRate(): number {
    const first = this.samples[0];
    const last = this.samples[this.samples.length - 1];
    return (last - first) / first;
  }

  private alertMemoryLeak(growth: number) {
    // Send alert to monitoring system
    console.error('MEMORY LEAK ALERT', {
      growthRate: growth,
      currentHeap: process.memoryUsage().heapUsed,
      timestamp: new Date().toISOString()
    });
  }

  stopMonitoring() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }
}
```

### High CPU Usage

**Optimization**:
```python
# cpu-optimization.py
import asyncio
import multiprocessing as mp
from concurrent.futures import ProcessPoolExecutor

class OptimizedDDEXProcessor:
    def __init__(self, max_workers=None):
        self.max_workers = max_workers or mp.cpu_count()
        
    async def process_files_parallel(self, file_paths):
        """Process files using multiple CPU cores"""
        
        with ProcessPoolExecutor(max_workers=self.max_workers) as executor:
            loop = asyncio.get_event_loop()
            
            # Create tasks for each file
            tasks = [
                loop.run_in_executor(executor, self.process_single_file, file_path)
                for file_path in file_paths
            ]
            
            # Wait for all tasks to complete
            results = await asyncio.gather(*tasks, return_exceptions=True)
            
        return results
    
    @staticmethod
    def process_single_file(file_path):
        """Process a single file (runs in separate process)"""
        from ddex_parser import DDEXParser
        
        parser = DDEXParser()
        
        with open(file_path, 'r') as f:
            return parser.parse(f.read())
```

## Monitoring and Alerting

### Application Metrics

```typescript
// metrics.ts
import { createPrometheusMetrics } from './prometheus';

const metrics = createPrometheusMetrics();

export class DDEXMetrics {
  recordProcessingTime(operation: string, duration: number) {
    metrics.processingDuration
      .labels({ operation })
      .observe(duration);
  }

  incrementCounter(metric: string, labels: Record<string, string> = {}) {
    metrics[metric]?.labels(labels).inc();
  }

  recordMemoryUsage() {
    const usage = process.memoryUsage();
    metrics.memoryUsage.set(usage.heapUsed);
  }

  recordError(operation: string, errorType: string) {
    metrics.errors
      .labels({ operation, errorType })
      .inc();
  }
}

// Usage in DDEX processing
const metricsCollector = new DDEXMetrics();

export async function processWithMetrics(xmlContent: string) {
  const startTime = Date.now();
  
  try {
    const result = await parser.parse(xmlContent);
    
    metricsCollector.recordProcessingTime(
      'parse', 
      Date.now() - startTime
    );
    
    metricsCollector.incrementCounter('ddex_processed', {
      status: 'success',
      version: result.version
    });
    
    return result;
    
  } catch (error) {
    metricsCollector.recordError('parse', error.constructor.name);
    metricsCollector.incrementCounter('ddex_processed', {
      status: 'error'
    });
    
    throw error;
  } finally {
    metricsCollector.recordMemoryUsage();
  }
}
```

## Troubleshooting Checklist

### Before Deployment
- [ ] All dependencies properly installed
- [ ] Environment variables configured
- [ ] Database connections tested
- [ ] Health checks implemented
- [ ] Monitoring configured
- [ ] Resource limits set
- [ ] Security settings applied

### After Deployment
- [ ] Application starts successfully
- [ ] Health checks passing
- [ ] Metrics being collected
- [ ] Logs are readable and useful
- [ ] Performance within expected ranges
- [ ] No memory leaks detected
- [ ] Error rates acceptable

### Common Commands
```bash
# Check process status
ps aux | grep ddex

# Monitor resource usage
top -p $(pgrep -f ddex-processor)

# Check logs
docker logs ddex-processor
kubectl logs deployment/ddex-processor

# Test health endpoint
curl http://localhost:3000/health

# Check memory usage
cat /proc/$(pgrep -f ddex-processor)/status | grep Vm
```

## Best Practices

1. **Use Health Checks**: Implement comprehensive health checks
2. **Monitor Resources**: Track CPU, memory, and disk usage
3. **Set Resource Limits**: Configure appropriate resource limits
4. **Implement Graceful Shutdown**: Handle shutdown signals properly
5. **Use Rolling Deployments**: Deploy with zero downtime
6. **Test in Staging**: Always test deployments in staging first
7. **Have Rollback Plan**: Be prepared to rollback if needed
8. **Monitor After Deployment**: Watch metrics closely after deployment