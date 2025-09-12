#!/usr/bin/env node

/**
 * DDEX Suite Node.js Performance Benchmark
 * Tests both parser and builder across multiple file sizes
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const { performance } = require('perf_hooks');
const { Worker, isMainThread, parentPort, workerData } = require('worker_threads');

// Import DDEX Suite components
let DDEXParser, DDEXBuilder;
try {
    // Try to import from installed packages first
    const parserPkg = require('ddex-parser');
    DDEXParser = parserPkg.DDEXParser || parserPkg.default || parserPkg;
    
    const builderPkg = require('ddex-builder');
    DDEXBuilder = builderPkg.DDEXBuilder || builderPkg.default || builderPkg;
    
    console.log('📦 Loaded from npm packages');
} catch (e) {
    console.log('📦 Using local bindings...');
    try {
        // Fallback to local packages
        const parserPkg = require('../../packages/ddex-parser/bindings/node');
        DDEXParser = parserPkg.DDEXParser || parserPkg.default || parserPkg;
        
        const builderPkg = require('../../packages/ddex-builder/bindings/node');
        DDEXBuilder = builderPkg.DDEXBuilder || builderPkg.default || builderPkg;
    } catch (e2) {
        console.error('❌ Could not load DDEX Suite bindings:', e2.message);
        process.exit(1);
    }
}

class NodejsBenchmarkSuite {
    constructor() {
        console.log('DDEXParser type:', typeof DDEXParser);
        console.log('DDEXBuilder type:', typeof DDEXBuilder);
        
        this.parser = new DDEXParser();
        this.builder = DDEXBuilder ? new DDEXBuilder() : null;
        this.results = [];
    }

    getMemoryUsage() {
        const mem = process.memoryUsage();
        return {
            rss: mem.rss,
            heapUsed: mem.heapUsed,
            heapTotal: mem.heapTotal,
            external: mem.external,
        };
    }

    async benchmarkFile(filePath) {
        const fileName = path.basename(filePath);
        const stats = fs.statSync(filePath);
        const fileSize = stats.size;
        
        console.log(`📊 Benchmarking ${fileName} (${(fileSize / 1024).toFixed(1)}KB)...`);

        const memoryBefore = this.getMemoryUsage();
        let result = {
            fileName,
            fileSize,
            parseTime: 0,
            buildTime: 0,
            memoryUsage: {
                before: memoryBefore,
                after: null,
                peak: memoryBefore
            },
            success: false,
            error: null
        };

        try {
            // Parse benchmark
            const parseStart = performance.now();
            const xmlContent = fs.readFileSync(filePath, 'utf-8');
            const parseResult = await this.parser.parse(xmlContent);
            const parseTime = performance.now() - parseStart;
            result.parseTime = parseTime;

            const memoryAfterParse = this.getMemoryUsage();
            result.memoryUsage.peak = {
                rss: Math.max(memoryBefore.rss, memoryAfterParse.rss),
                heapUsed: Math.max(memoryBefore.heapUsed, memoryAfterParse.heapUsed)
            };

            // Build benchmark (if available)
            let buildTime = 0;
            try {
                if (this.builder && parseResult && parseResult.flat) {
                    const buildStart = performance.now();
                    // Create a simple build request from parsed data
                    const buildRequest = {
                        releases: parseResult.flat.releases || [],
                        resources: parseResult.flat.resources || [],
                        deals: parseResult.flat.deals || []
                    };
                    const buildResult = await this.builder.build(buildRequest);
                    buildTime = performance.now() - buildStart;
                    result.buildTime = buildTime;
                }
            } catch (buildError) {
                // Build might fail due to API differences, but that's okay for performance testing
                console.log(`  ⚠️  Build skipped: ${buildError.message}`);
            }

            result.memoryUsage.after = this.getMemoryUsage();
            result.success = true;

            const memUsageMB = (result.memoryUsage.peak.heapUsed - memoryBefore.heapUsed) / (1024 * 1024);
            console.log(`  ✅ Parse: ${parseTime.toFixed(2)}ms, Build: ${buildTime.toFixed(2)}ms, Memory: ${memUsageMB.toFixed(1)}MB`);

        } catch (error) {
            result.error = error.message;
            result.memoryUsage.after = this.getMemoryUsage();
            console.log(`  ❌ Failed: ${error.message}`);
        }

        return result;
    }

    async runParallelBenchmark(filePath, numWorkers) {
        console.log(`🚀 Testing parallel processing with ${numWorkers} workers...`);
        
        const xmlContent = fs.readFileSync(filePath, 'utf-8');
        const workerPromises = [];
        
        const start = performance.now();
        
        for (let i = 0; i < numWorkers; i++) {
            const workerPromise = new Promise((resolve, reject) => {
                const worker = new Worker(__filename, {
                    workerData: { xmlContent, workerId: i }
                });
                
                const workerStart = performance.now();
                worker.on('message', (result) => {
                    if (result.success) {
                        resolve(performance.now() - workerStart);
                    } else {
                        resolve(0);
                    }
                });
                
                worker.on('error', () => resolve(0));
                worker.on('exit', (code) => {
                    if (code !== 0) resolve(0);
                });
            });
            
            workerPromises.push(workerPromise);
        }
        
        const results = await Promise.all(workerPromises);
        const totalTime = performance.now() - start;
        const avgTime = results.filter(t => t > 0).reduce((a, b) => a + b, 0) / results.length;
        
        console.log(`  Total time: ${totalTime.toFixed(2)}ms, Average per worker: ${avgTime.toFixed(2)}ms`);
        return results;
    }

    generateReport() {
        console.log('\n🎯 NODE.JS PERFORMANCE BENCHMARK REPORT');
        console.log('='.repeat(80));
        
        let successfulParses = 0;
        let totalParseTime = 0;
        let totalBuildTime = 0;
        let totalMemoryUsage = 0;

        console.log(
            'File'.padEnd(15) +
            'Size'.padStart(8) +
            'Parse(ms)'.padStart(12) +
            'Build(ms)'.padStart(12) +
            'Memory(MB)'.padStart(12) +
            'Status'.padStart(10)
        );
        console.log('-'.repeat(80));

        for (const result of this.results) {
            const sizeStr = result.fileSize > 1024 * 1024 
                ? `${(result.fileSize / (1024 * 1024)).toFixed(1)}MB`
                : `${Math.round(result.fileSize / 1024)}KB`;
            
            const memUsage = result.memoryUsage.after && result.memoryUsage.before
                ? (result.memoryUsage.after.heapUsed - result.memoryUsage.before.heapUsed) / (1024 * 1024)
                : 0;

            const status = result.success ? '✅' : '❌';
            
            console.log(
                result.fileName.padEnd(15) +
                sizeStr.padStart(8) +
                result.parseTime.toFixed(2).padStart(12) +
                result.buildTime.toFixed(2).padStart(12) +
                memUsage.toFixed(1).padStart(12) +
                status.padStart(10)
            );

            if (result.success) {
                successfulParses++;
                totalParseTime += result.parseTime;
                totalBuildTime += result.buildTime;
                totalMemoryUsage += memUsage;
            }
        }

        console.log('-'.repeat(80));
        console.log('📈 PERFORMANCE SUMMARY:');
        console.log(`  Successful parses: ${successfulParses}/${this.results.length}`);

        if (successfulParses > 0) {
            const avgParse = totalParseTime / successfulParses;
            const avgBuild = totalBuildTime / successfulParses;
            const avgMemory = totalMemoryUsage / successfulParses;
            
            console.log(`  Average parse time: ${avgParse.toFixed(2)}ms`);
            console.log(`  Average build time: ${avgBuild.toFixed(2)}ms`);
            console.log(`  Average memory usage: ${avgMemory.toFixed(1)}MB`);
        }

        // Check targets
        const largeFileResults = this.results.filter(r => r.fileSize >= 10 * 1024 * 1024 && r.success);
        if (largeFileResults.length > 0) {
            const avg10mbParse = largeFileResults.reduce((sum, r) => sum + r.parseTime, 0) / largeFileResults.length;
            console.log(`  Average 10MB+ parse time: ${avg10mbParse.toFixed(2)}ms (target: <50ms) ${avg10mbParse < 50 ? '✅' : '❌'}`);
        }

        // Performance throughput
        const totalData = this.results.filter(r => r.success).reduce((sum, r) => sum + r.fileSize, 0);
        const totalTime = totalParseTime / 1000; // Convert to seconds
        if (totalTime > 0) {
            const throughputMBs = (totalData / (1024 * 1024)) / totalTime;
            console.log(`  Overall throughput: ${throughputMBs.toFixed(1)} MB/s`);
        }
    }

    async run() {
        console.log('🚀 DDEX Suite Node.js Performance Benchmark');
        console.log('='.repeat(60));

        // Test files
        const testFiles = [
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
        ].map(f => path.join(__dirname, f));

        console.log('\n📊 Testing parse performance...');
        for (const filePath of testFiles) {
            if (fs.existsSync(filePath)) {
                const result = await this.benchmarkFile(filePath);
                this.results.push(result);
            }
        }

        // Parallel processing test
        const mediumFile = testFiles.find(f => fs.existsSync(f) && f.includes('1mb'));
        if (mediumFile) {
            console.log('\n🔄 Testing parallel processing capabilities...');
            await this.runParallelBenchmark(mediumFile, 4);
            await this.runParallelBenchmark(mediumFile, 8);
        }

        // Generate report
        this.generateReport();
        
        console.log('\n🎉 Node.js benchmark completed!');
    }
}

// Worker thread handler
if (!isMainThread) {
    const { xmlContent, workerId } = workerData;
    
    (async () => {
        try {
            const parser = new DDEXParser();
            const parseResult = await parser.parse(xmlContent);
            parentPort.postMessage({ success: true, workerId });
        } catch (error) {
            parentPort.postMessage({ success: false, error: error.message, workerId });
        }
    })();
} else {
    // Main thread - run benchmark
    const suite = new NodejsBenchmarkSuite();
    suite.run().catch(console.error);
}