# Performance Monitoring

Monitor and optimize DDEX Suite performance for production environments.

## Overview

Performance monitoring helps:
- Identify bottlenecks in processing pipelines
- Track resource usage patterns
- Optimize for large-scale deployments
- Ensure SLA compliance
- Plan capacity requirements

## Performance Metrics

### Core Metrics

```typescript
import { performance } from 'perf_hooks';
import { DDEXParser, DDEXBuilder } from 'ddex-suite';

export class PerformanceMonitor {
  private metrics: Map<string, PerformanceMetric[]> = new Map();

  recordMetric(name: string, value: number, tags: Record<string, string> = {}): void {
    if (!this.metrics.has(name)) {
      this.metrics.set(name, []);
    }
    
    this.metrics.get(name)!.push({
      timestamp: Date.now(),
      value,
      tags
    });
  }

  async measureOperation<T>(
    name: string,
    operation: () => Promise<T>,
    tags: Record<string, string> = {}
  ): Promise<T> {
    const startTime = performance.now();
    const startMemory = process.memoryUsage();
    
    try {
      const result = await operation();
      const endTime = performance.now();
      const endMemory = process.memoryUsage();
      
      // Record timing
      this.recordMetric(`${name}.duration`, endTime - startTime, tags);
      
      // Record memory usage
      this.recordMetric(`${name}.memory_delta`, 
        endMemory.heapUsed - startMemory.heapUsed, tags);
      
      // Record success
      this.recordMetric(`${name}.success`, 1, tags);
      
      return result;
      
    } catch (error) {
      const endTime = performance.now();
      
      // Record failure timing
      this.recordMetric(`${name}.duration`, endTime - startTime, { ...tags, status: 'error' });
      this.recordMetric(`${name}.error`, 1, { ...tags, error_type: error.constructor.name });
      
      throw error;
    }
  }

  getMetrics(name: string, timeRange?: { start: number, end: number }): PerformanceMetric[] {
    const metrics = this.metrics.get(name) || [];
    
    if (!timeRange) {
      return metrics;
    }
    
    return metrics.filter(m => 
      m.timestamp >= timeRange.start && m.timestamp <= timeRange.end
    );
  }

  getAverageValue(name: string, timeRange?: { start: number, end: number }): number {
    const metrics = this.getMetrics(name, timeRange);
    if (metrics.length === 0) return 0;
    
    return metrics.reduce((sum, m) => sum + m.value, 0) / metrics.length;
  }

  getPercentile(name: string, percentile: number, timeRange?: { start: number, end: number }): number {
    const metrics = this.getMetrics(name, timeRange);
    if (metrics.length === 0) return 0;
    
    const values = metrics.map(m => m.value).sort((a, b) => a - b);
    const index = Math.ceil((percentile / 100) * values.length) - 1;
    
    return values[Math.max(0, index)];
  }
}

interface PerformanceMetric {
  timestamp: number;
  value: number;
  tags: Record<string, string>;
}

// Usage with DDEX operations
const monitor = new PerformanceMonitor();
const parser = new DDEXParser();
const builder = new DDEXBuilder();

// Monitored parsing
const result = await monitor.measureOperation(
  'ddex.parse',
  () => parser.parse(xmlContent),
  { 
    file_size: String(xmlContent.length),
    ddex_version: '4.3'
  }
);

// Monitored building
const xml = await monitor.measureOperation(
  'ddex.build',
  () => builder.build(data),
  {
    release_count: String(data.releases.length),
    validation_level: 'strict'
  }
);
```

### Advanced Profiling

```python
import time
import psutil
import threading
from contextlib import contextmanager
from typing import Dict, List, Optional
from dataclasses import dataclass
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

@dataclass
class PerformanceProfile:
    operation: str
    start_time: float
    end_time: float
    cpu_percent: float
    memory_mb: float
    memory_peak_mb: float
    file_size: Optional[int] = None
    records_processed: Optional[int] = None

class DDEXProfiler:
    def __init__(self):
        self.parser = DDEXParser()
        self.builder = DDEXBuilder()
        self.profiles: List[PerformanceProfile] = []
        self.process = psutil.Process()
    
    @contextmanager
    def profile_operation(self, operation: str, **metadata):
        """Context manager for profiling DDEX operations"""
        
        # Start monitoring
        start_time = time.time()
        start_memory = self.process.memory_info().rss / 1024 / 1024  # MB
        
        # Start CPU monitoring in background thread
        cpu_samples = []
        monitoring = threading.Event()
        
        def monitor_cpu():
            while not monitoring.is_set():
                cpu_samples.append(self.process.cpu_percent(interval=0.1))
                time.sleep(0.1)
        
        monitor_thread = threading.Thread(target=monitor_cpu)
        monitor_thread.start()
        
        peak_memory = start_memory
        
        try:
            yield
            
        finally:
            # Stop monitoring
            end_time = time.time()
            monitoring.set()
            monitor_thread.join()
            
            end_memory = self.process.memory_info().rss / 1024 / 1024  # MB
            avg_cpu = sum(cpu_samples) / len(cpu_samples) if cpu_samples else 0
            
            # Record profile
            profile = PerformanceProfile(
                operation=operation,
                start_time=start_time,
                end_time=end_time,
                cpu_percent=avg_cpu,
                memory_mb=end_memory - start_memory,
                memory_peak_mb=peak_memory - start_memory,
                **metadata
            )
            
            self.profiles.append(profile)
            
            # Log performance data
            duration = end_time - start_time
            print(f"Performance: {operation} took {duration:.3f}s, "
                  f"CPU: {avg_cpu:.1f}%, Memory: {end_memory - start_memory:.1f}MB")
    
    def parse_with_profiling(self, xml_content: str) -> any:
        """Parse DDEX with performance profiling"""
        
        with self.profile_operation(
            "parse",
            file_size=len(xml_content)
        ):
            result = self.parser.parse(xml_content)
            
            # Update profile with parsed data info
            if self.profiles:
                self.profiles[-1].records_processed = len(result.flat.releases)
            
            return result
    
    def build_with_profiling(self, data: dict, **options) -> str:
        """Build DDEX with performance profiling"""
        
        release_count = len(data.get('releases', []))
        
        with self.profile_operation(
            "build", 
            records_processed=release_count
        ):
            return self.builder.build(data, **options)
    
    def get_performance_summary(self) -> Dict:
        """Generate performance summary statistics"""
        
        if not self.profiles:
            return {}
        
        # Group by operation
        by_operation = {}
        for profile in self.profiles:
            if profile.operation not in by_operation:
                by_operation[profile.operation] = []
            by_operation[profile.operation].append(profile)
        
        summary = {}
        for operation, profiles in by_operation.items():
            durations = [p.end_time - p.start_time for p in profiles]
            cpu_usage = [p.cpu_percent for p in profiles]
            memory_usage = [p.memory_mb for p in profiles]
            
            summary[operation] = {
                'count': len(profiles),
                'duration': {
                    'avg': sum(durations) / len(durations),
                    'min': min(durations),
                    'max': max(durations),
                    'p95': sorted(durations)[int(0.95 * len(durations))] if durations else 0
                },
                'cpu_percent': {
                    'avg': sum(cpu_usage) / len(cpu_usage),
                    'max': max(cpu_usage)
                },
                'memory_mb': {
                    'avg': sum(memory_usage) / len(memory_usage),
                    'max': max(memory_usage)
                }
            }
        
        return summary
    
    def export_profiles(self, filename: str):
        """Export performance profiles to CSV"""
        
        import csv
        
        with open(filename, 'w', newline='') as csvfile:
            fieldnames = [
                'operation', 'duration', 'cpu_percent', 'memory_mb', 
                'memory_peak_mb', 'file_size', 'records_processed'
            ]
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            
            writer.writeheader()
            for profile in self.profiles:
                writer.writerow({
                    'operation': profile.operation,
                    'duration': profile.end_time - profile.start_time,
                    'cpu_percent': profile.cpu_percent,
                    'memory_mb': profile.memory_mb,
                    'memory_peak_mb': profile.memory_peak_mb,
                    'file_size': profile.file_size,
                    'records_processed': profile.records_processed
                })

# Usage example
profiler = DDEXProfiler()

# Profile parsing
with open('large_release.xml', 'r') as f:
    content = f.read()
    result = profiler.parse_with_profiling(content)

# Profile building
data = result.to_build_request()
xml = profiler.build_with_profiling(data)

# Get summary
summary = profiler.get_performance_summary()
print("Performance Summary:", summary)

# Export detailed profiles
profiler.export_profiles('ddex_performance.csv')
```

## Resource Usage Monitoring

```typescript
export class ResourceMonitor {
  private readonly CHECK_INTERVAL = 1000; // 1 second
  private monitoring = false;
  private resourceData: ResourceSnapshot[] = [];

  startMonitoring(): void {
    if (this.monitoring) return;
    
    this.monitoring = true;
    this.collectResourceData();
  }

  stopMonitoring(): ResourceSummary {
    this.monitoring = false;
    return this.generateSummary();
  }

  private async collectResourceData(): Promise<void> {
    while (this.monitoring) {
      const snapshot = await this.takeSnapshot();
      this.resourceData.push(snapshot);
      
      // Keep only last 5 minutes of data
      const fiveMinutesAgo = Date.now() - 5 * 60 * 1000;
      this.resourceData = this.resourceData.filter(s => s.timestamp > fiveMinutesAgo);
      
      await new Promise(resolve => setTimeout(resolve, this.CHECK_INTERVAL));
    }
  }

  private async takeSnapshot(): Promise<ResourceSnapshot> {
    const memUsage = process.memoryUsage();
    const cpuUsage = process.cpuUsage();
    
    return {
      timestamp: Date.now(),
      memory: {
        heapUsed: memUsage.heapUsed,
        heapTotal: memUsage.heapTotal,
        external: memUsage.external,
        rss: memUsage.rss
      },
      cpu: {
        user: cpuUsage.user,
        system: cpuUsage.system
      },
      handles: process._getActiveHandles().length,
      requests: process._getActiveRequests().length
    };
  }

  private generateSummary(): ResourceSummary {
    if (this.resourceData.length === 0) {
      throw new Error('No resource data collected');
    }

    const memoryValues = this.resourceData.map(s => s.memory.heapUsed);
    const cpuValues = this.resourceData.map(s => s.cpu.user + s.cpu.system);

    return {
      duration: this.resourceData[this.resourceData.length - 1].timestamp - this.resourceData[0].timestamp,
      memory: {
        min: Math.min(...memoryValues),
        max: Math.max(...memoryValues),
        avg: memoryValues.reduce((a, b) => a + b, 0) / memoryValues.length
      },
      cpu: {
        min: Math.min(...cpuValues),
        max: Math.max(...cpuValues),
        avg: cpuValues.reduce((a, b) => a + b, 0) / cpuValues.length
      },
      samplesCollected: this.resourceData.length
    };
  }
}

interface ResourceSnapshot {
  timestamp: number;
  memory: {
    heapUsed: number;
    heapTotal: number;
    external: number;
    rss: number;
  };
  cpu: {
    user: number;
    system: number;
  };
  handles: number;
  requests: number;
}

interface ResourceSummary {
  duration: number;
  memory: { min: number; max: number; avg: number };
  cpu: { min: number; max: number; avg: number };
  samplesCollected: number;
}

// Usage in DDEX processing
export class MonitoredDDEXService {
  private parser = new DDEXParser();
  private builder = new DDEXBuilder();
  private resourceMonitor = new ResourceMonitor();

  async processWithMonitoring(xmlContent: string): Promise<{
    result: any;
    performance: ResourceSummary;
  }> {
    this.resourceMonitor.startMonitoring();
    
    try {
      const result = await this.parser.parse(xmlContent);
      const performance = this.resourceMonitor.stopMonitoring();
      
      return { result, performance };
      
    } catch (error) {
      this.resourceMonitor.stopMonitoring();
      throw error;
    }
  }
}
```

## Performance Benchmarking

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ddex_parser::DDEXParser;
use ddex_builder::DDEXBuilder;
use std::time::Duration;

fn benchmark_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("ddex_parsing");
    
    // Different file sizes
    let test_files = vec![
        ("small_1kb", include_str!("../test_data/small_release.xml")),
        ("medium_10kb", include_str!("../test_data/medium_release.xml")),
        ("large_100kb", include_str!("../test_data/large_release.xml")),
        ("xlarge_1mb", include_str!("../test_data/xlarge_release.xml")),
    ];
    
    for (name, content) in test_files {
        group.bench_with_input(
            BenchmarkId::new("parse", name),
            &content,
            |b, &content| {
                let parser = DDEXParser::new();
                b.iter(|| {
                    let result = parser.parse(black_box(content));
                    black_box(result)
                })
            }
        );
    }
    
    group.finish();
}

fn benchmark_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("ddex_building");
    
    // Different data complexities
    let test_data = vec![
        ("simple", create_simple_release_data()),
        ("complex", create_complex_release_data()),
        ("multi_release", create_multi_release_data()),
    ];
    
    for (name, data) in test_data {
        group.bench_with_input(
            BenchmarkId::new("build", name),
            &data,
            |b, data| {
                let builder = DDEXBuilder::new();
                b.iter(|| {
                    let result = builder.build(black_box(data.clone()));
                    black_box(result)
                })
            }
        );
    }
    
    group.finish();
}

fn benchmark_round_trip(c: &mut Criterion) {
    let mut group = c.benchmark_group("ddex_round_trip");
    group.measurement_time(Duration::from_secs(10));
    
    let xml_content = include_str!("../test_data/sample_release.xml");
    
    group.bench_function("parse_build_parse", |b| {
        let parser = DDEXParser::new();
        let builder = DDEXBuilder::new();
        
        b.iter(|| {
            // Parse
            let parsed = parser.parse(black_box(xml_content)).unwrap();
            
            // Build
            let rebuilt = builder.build(black_box(&parsed.to_build_request())).unwrap();
            
            // Parse again
            let reparsed = parser.parse(black_box(&rebuilt)).unwrap();
            
            black_box(reparsed)
        })
    });
    
    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // Benchmark memory-intensive operations
    group.bench_function("large_file_streaming", |b| {
        let large_xml = create_large_xml_content(); // 10MB+
        let parser = DDEXParser::new_with_streaming();
        
        b.iter(|| {
            let result = parser.parse_streaming(black_box(&large_xml));
            black_box(result)
        })
    });
    
    group.finish();
}

// Performance regression tests
fn performance_regression_tests(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_tests");
    
    // Set baseline expectations
    group.bench_function("parse_10kb_baseline", |b| {
        let content = include_str!("../test_data/10kb_release.xml");
        let parser = DDEXParser::new();
        
        b.iter(|| {
            let start = std::time::Instant::now();
            let result = parser.parse(black_box(content));
            let duration = start.elapsed();
            
            // Assert performance requirement: < 10ms for 10KB file
            assert!(duration < Duration::from_millis(10), 
                "Performance regression: 10KB parse took {:?}", duration);
            
            black_box(result)
        })
    });
    
    group.bench_function("build_100_releases_baseline", |b| {
        let data = create_100_release_data();
        let builder = DDEXBuilder::new();
        
        b.iter(|| {
            let start = std::time::Instant::now();
            let result = builder.build(black_box(&data));
            let duration = start.elapsed();
            
            // Assert performance requirement: < 1s for 100 releases
            assert!(duration < Duration::from_secs(1),
                "Performance regression: 100 releases build took {:?}", duration);
            
            black_box(result)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches, 
    benchmark_parsing, 
    benchmark_building, 
    benchmark_round_trip,
    benchmark_memory_usage,
    performance_regression_tests
);

criterion_main!(benches);

// Helper functions
fn create_simple_release_data() -> serde_json::Value {
    serde_json::json!({
        "releases": [{
            "id": "R001",
            "title": "Test Album",
            "artist": "Test Artist",
            "tracks": [{
                "id": "T001",
                "title": "Test Track",
                "artist": "Test Artist",
                "duration_ms": 180000
            }]
        }]
    })
}

fn create_complex_release_data() -> serde_json::Value {
    // Create complex data with multiple releases, tracks, metadata, etc.
    serde_json::json!({
        "releases": (0..5).map(|i| serde_json::json!({
            "id": format!("R{:03}", i),
            "title": format!("Album {}", i),
            "artist": format!("Artist {}", i),
            "tracks": (0..12).map(|j| serde_json::json!({
                "id": format!("T{:03}{:03}", i, j),
                "title": format!("Track {} - {}", i, j),
                "artist": format!("Artist {}", i),
                "duration_ms": 180000 + j * 1000,
                "isrc": format!("US{:02}{:08}", i, j),
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}
```

## Performance Alerting

```python
import smtplib
from email.mime.text import MimeText
from dataclasses import dataclass
from typing import List, Callable
import json

@dataclass
class PerformanceThreshold:
    metric: str
    threshold: float
    comparison: str  # 'gt', 'lt', 'eq'
    description: str

@dataclass
class PerformanceAlert:
    timestamp: float
    metric: str
    value: float
    threshold: float
    severity: str
    description: str

class PerformanceAlerting:
    def __init__(self, smtp_config: dict):
        self.smtp_config = smtp_config
        self.thresholds: List[PerformanceThreshold] = []
        self.alert_handlers: List[Callable[[PerformanceAlert], None]] = []
        
        # Default thresholds
        self.add_threshold('parse.duration', 5.0, 'gt', 'Parse duration > 5s')
        self.add_threshold('build.duration', 10.0, 'gt', 'Build duration > 10s')
        self.add_threshold('memory.usage', 1000, 'gt', 'Memory usage > 1GB')
        self.add_threshold('error.rate', 0.05, 'gt', 'Error rate > 5%')
    
    def add_threshold(self, metric: str, threshold: float, comparison: str, description: str):
        """Add performance threshold"""
        self.thresholds.append(PerformanceThreshold(
            metric=metric,
            threshold=threshold,
            comparison=comparison,
            description=description
        ))
    
    def add_alert_handler(self, handler: Callable[[PerformanceAlert], None]):
        """Add custom alert handler"""
        self.alert_handlers.append(handler)
    
    def check_thresholds(self, metrics: dict):
        """Check metrics against thresholds and trigger alerts"""
        
        for threshold in self.thresholds:
            if threshold.metric in metrics:
                value = metrics[threshold.metric]
                
                should_alert = False
                if threshold.comparison == 'gt' and value > threshold.threshold:
                    should_alert = True
                elif threshold.comparison == 'lt' and value < threshold.threshold:
                    should_alert = True
                elif threshold.comparison == 'eq' and value == threshold.threshold:
                    should_alert = True
                
                if should_alert:
                    alert = PerformanceAlert(
                        timestamp=time.time(),
                        metric=threshold.metric,
                        value=value,
                        threshold=threshold.threshold,
                        severity='warning' if value < threshold.threshold * 2 else 'critical',
                        description=threshold.description
                    )
                    
                    self.trigger_alert(alert)
    
    def trigger_alert(self, alert: PerformanceAlert):
        """Trigger performance alert"""
        
        # Built-in email handler
        self.send_email_alert(alert)
        
        # Custom handlers
        for handler in self.alert_handlers:
            try:
                handler(alert)
            except Exception as e:
                print(f"Alert handler failed: {e}")
    
    def send_email_alert(self, alert: PerformanceAlert):
        """Send email alert"""
        
        subject = f"DDEX Performance Alert: {alert.metric}"
        
        body = f"""
        Performance Alert Triggered
        
        Metric: {alert.metric}
        Value: {alert.value}
        Threshold: {alert.threshold}
        Severity: {alert.severity}
        Description: {alert.description}
        Timestamp: {alert.timestamp}
        
        Please investigate the performance issue.
        """
        
        msg = MimeText(body)
        msg['Subject'] = subject
        msg['From'] = self.smtp_config['from']
        msg['To'] = self.smtp_config['to']
        
        try:
            with smtplib.SMTP(self.smtp_config['host'], self.smtp_config['port']) as server:
                if self.smtp_config.get('username'):
                    server.starttls()
                    server.login(self.smtp_config['username'], self.smtp_config['password'])
                
                server.send_message(msg)
                
        except Exception as e:
            print(f"Failed to send email alert: {e}")

# Integration with monitoring
def slack_alert_handler(alert: PerformanceAlert):
    """Send alert to Slack"""
    import requests
    
    webhook_url = "https://hooks.slack.com/your-webhook-url"
    
    message = {
        "text": f"🚨 Performance Alert: {alert.description}",
        "attachments": [
            {
                "color": "danger" if alert.severity == 'critical' else "warning",
                "fields": [
                    {"title": "Metric", "value": alert.metric, "short": True},
                    {"title": "Value", "value": str(alert.value), "short": True},
                    {"title": "Threshold", "value": str(alert.threshold), "short": True},
                    {"title": "Severity", "value": alert.severity, "short": True}
                ]
            }
        ]
    }
    
    requests.post(webhook_url, json=message)

# Usage
alerting = PerformanceAlerting({
    'host': 'smtp.gmail.com',
    'port': 587,
    'username': 'alerts@yourcompany.com',
    'password': 'your-password',
    'from': 'alerts@yourcompany.com',
    'to': 'team@yourcompany.com'
})

alerting.add_alert_handler(slack_alert_handler)

# Check performance metrics
current_metrics = {
    'parse.duration': 7.5,  # This will trigger an alert
    'build.duration': 3.2,
    'memory.usage': 500,
    'error.rate': 0.02
}

alerting.check_thresholds(current_metrics)
```

## Best Practices

1. **Baseline Establishment**: Establish performance baselines for your workload
2. **Continuous Monitoring**: Monitor performance continuously in production
3. **Alerting Thresholds**: Set appropriate alerting thresholds for critical metrics
4. **Profiling**: Use profiling tools to identify bottlenecks
5. **Benchmarking**: Run regular benchmarks to detect performance regressions
6. **Resource Planning**: Use performance data for capacity planning
7. **Optimization**: Continuously optimize based on performance data
8. **Documentation**: Document performance characteristics and requirements
9. **Testing**: Include performance tests in your CI/CD pipeline
10. **Monitoring Overhead**: Keep monitoring overhead minimal