#!/usr/bin/env python3
"""
DDEX Suite Python Performance Benchmark
Tests parser performance across multiple file sizes
"""

import os
import sys
import time
import psutil
import threading
import concurrent.futures
from pathlib import Path

# Try to import DDEX Parser
try:
    import ddex_parser
    print("📦 Loaded ddex-parser from PyPI")
except ImportError:
    try:
        # Fallback to local development build
        sys.path.insert(0, str(Path(__file__).parent.parent.parent / "packages" / "ddex-parser" / "bindings" / "python"))
        import ddex_parser
        print("📦 Loaded ddex-parser from local bindings")
    except ImportError:
        print("❌ Could not load ddex-parser")
        sys.exit(1)

class PythonBenchmarkSuite:
    def __init__(self):
        self.parser = ddex_parser.DDEXParser()
        self.results = []
        self.process = psutil.Process()

    def get_memory_usage(self):
        """Get current memory usage in bytes"""
        memory_info = self.process.memory_info()
        return {
            'rss': memory_info.rss,
            'vms': memory_info.vms,
        }

    def benchmark_file(self, file_path):
        """Benchmark parsing a single file"""
        file_name = os.path.basename(file_path)
        file_size = os.path.getsize(file_path)
        
        print(f"📊 Benchmarking {file_name} ({file_size / 1024:.1f}KB)...")

        memory_before = self.get_memory_usage()
        result = {
            'file_name': file_name,
            'file_size': file_size,
            'parse_time': 0,
            'memory_usage': {
                'before': memory_before,
                'after': None,
                'peak': memory_before
            },
            'success': False,
            'error': None
        }

        try:
            # Parse benchmark
            with open(file_path, 'r', encoding='utf-8') as f:
                xml_content = f.read()
            
            start_time = time.perf_counter()
            parse_result = self.parser.parse(xml_content)
            end_time = time.perf_counter()
            parse_time = (end_time - start_time) * 1000  # Convert to milliseconds
            result['parse_time'] = parse_time

            memory_after = self.get_memory_usage()
            result['memory_usage']['after'] = memory_after
            result['memory_usage']['peak'] = {
                'rss': max(memory_before['rss'], memory_after['rss']),
                'vms': max(memory_before['vms'], memory_after['vms'])
            }
            
            result['success'] = True

            mem_usage_mb = (memory_after['rss'] - memory_before['rss']) / (1024 * 1024)
            print(f"  ✅ Parse: {parse_time:.2f}ms, Memory: {mem_usage_mb:.1f}MB")

        except Exception as error:
            result['error'] = str(error)
            result['memory_usage']['after'] = self.get_memory_usage()
            print(f"  ❌ Failed: {error}")

        return result

    def run_parallel_benchmark(self, file_path, num_workers):
        """Test parallel processing with multiple workers"""
        print(f"🚀 Testing parallel processing with {num_workers} workers...")
        
        with open(file_path, 'r', encoding='utf-8') as f:
            xml_content = f.read()
        
        def parse_worker(worker_id):
            try:
                parser = ddex_parser.DDEXParser()
                start_time = time.perf_counter()
                result = parser.parse(xml_content)
                end_time = time.perf_counter()
                return (end_time - start_time) * 1000
            except Exception:
                return 0

        start_time = time.perf_counter()
        
        with concurrent.futures.ThreadPoolExecutor(max_workers=num_workers) as executor:
            futures = [executor.submit(parse_worker, i) for i in range(num_workers)]
            results = [future.result() for future in concurrent.futures.as_completed(futures)]
        
        total_time = (time.perf_counter() - start_time) * 1000
        avg_time = sum(r for r in results if r > 0) / len([r for r in results if r > 0])
        
        print(f"  Total time: {total_time:.2f}ms, Average per worker: {avg_time:.2f}ms")
        return results

    def generate_report(self):
        """Generate performance report"""
        print('\n🎯 PYTHON PERFORMANCE BENCHMARK REPORT')
        print('=' * 80)
        
        successful_parses = 0
        total_parse_time = 0
        total_memory_usage = 0

        print(f"{'File':<15} {'Size':>8} {'Parse(ms)':>12} {'Memory(MB)':>12} {'Status':>10}")
        print('-' * 67)

        for result in self.results:
            if result['file_size'] > 1024 * 1024:
                size_str = f"{result['file_size'] / (1024 * 1024):.1f}MB"
            else:
                size_str = f"{result['file_size'] // 1024}KB"
            
            mem_usage = 0
            if result['memory_usage']['after'] and result['memory_usage']['before']:
                mem_usage = (result['memory_usage']['after']['rss'] - 
                           result['memory_usage']['before']['rss']) / (1024 * 1024)

            status = '✅' if result['success'] else '❌'
            
            print(f"{result['file_name']:<15} {size_str:>8} {result['parse_time']:>12.2f} "
                  f"{mem_usage:>12.1f} {status:>10}")

            if result['success']:
                successful_parses += 1
                total_parse_time += result['parse_time']
                total_memory_usage += mem_usage

        print('-' * 67)
        print('📈 PERFORMANCE SUMMARY:')
        print(f"  Successful parses: {successful_parses}/{len(self.results)}")

        if successful_parses > 0:
            avg_parse = total_parse_time / successful_parses
            avg_memory = total_memory_usage / successful_parses
            
            print(f"  Average parse time: {avg_parse:.2f}ms")
            print(f"  Average memory usage: {avg_memory:.1f}MB")

        # Check targets
        large_file_results = [r for r in self.results if r['file_size'] >= 10 * 1024 * 1024 and r['success']]
        if large_file_results:
            avg_10mb_parse = sum(r['parse_time'] for r in large_file_results) / len(large_file_results)
            status = '✅' if avg_10mb_parse < 50 else '❌'
            print(f"  Average 10MB+ parse time: {avg_10mb_parse:.2f}ms (target: <50ms) {status}")

        # Performance throughput
        total_data = sum(r['file_size'] for r in self.results if r['success'])
        total_time_sec = total_parse_time / 1000  # Convert to seconds
        if total_time_sec > 0:
            throughput_mbs = (total_data / (1024 * 1024)) / total_time_sec
            print(f"  Overall throughput: {throughput_mbs:.1f} MB/s")

    def run(self):
        """Run the benchmark suite"""
        print('🚀 DDEX Suite Python Performance Benchmark')
        print('=' * 60)

        # Test files
        test_files = [
            '../test-data/1kb.xml',
            '../test-data/5kb.xml',
            '../test-data/10kb.xml',
            '../test-data/50kb.xml',
            '../test-data/100kb.xml',
            '../test-data/500kb.xml',
            '../test-data/1mb.xml',
            '../test-data/5mb.xml',
            '../test-data/10mb.xml',
            '../test-data/25mb.xml',
        ]

        print('\n📊 Testing parse performance...')
        for file_name in test_files:
            file_path = Path(__file__).parent / file_name
            if file_path.exists():
                result = self.benchmark_file(file_path)
                self.results.append(result)

        # Parallel processing test
        medium_file = None
        for file_name in test_files:
            file_path = Path(__file__).parent / file_name
            if file_path.exists() and '1mb' in file_name:
                medium_file = file_path
                break

        if medium_file:
            print('\n🔄 Testing parallel processing capabilities...')
            self.run_parallel_benchmark(medium_file, 4)
            self.run_parallel_benchmark(medium_file, 8)

        # Generate report
        self.generate_report()
        
        print('\n🎉 Python benchmark completed!')

if __name__ == '__main__':
    suite = PythonBenchmarkSuite()
    suite.run()