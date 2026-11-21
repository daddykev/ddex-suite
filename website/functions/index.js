/**
 * DDEX Suite API - Cloud Functions for Firebase
 * Provides REST API endpoints for DDEX parsing and building functionality
 */

const {setGlobalOptions} = require("firebase-functions");
const {onRequest} = require("firebase-functions/v2/https");
const logger = require("firebase-functions/logger");
const cors = require("cors");

// Import DDEX handlers
const { parseHandler, buildHandler, batchHandler } = require('./ddex-handlers');

// Initialize DDEX modules with proper detection
let DdexParser, DdexBuilder;
let nativeModulesAvailable = false;
let parserType = 'unknown';

// Simple XML parser implementation as fallback with extensive logging
class SimpleDdexParser {
  parse(xmlContent) {
    // Log the actual XML content we're parsing
    logger.info('XML SAMPLE', {
      first500: xmlContent.substring(0, 500).replace(/\n/g, ' ').replace(/\s+/g, ' ')
    });

    logger.info('SimpleDdexParser.parse START', {
      contentLength: xmlContent.length
    });

    // Log what we're searching for
    const messageIdMatch = xmlContent.match(/<MessageId[^>]*>([^<]+)<\/MessageId>/);
    logger.info('MessageId search', {
      found: !!messageIdMatch,
      value: messageIdMatch ? messageIdMatch[1] : null,
      rawMatch: messageIdMatch ? messageIdMatch[0] : null
    });

    const releaseMatches = xmlContent.match(/<Release[^>]*>/g) || [];
    logger.info('Release tags found', {
      count: releaseMatches.length,
      matches: releaseMatches.slice(0, 3) // First 3 for debugging
    });

    const trackMatches = xmlContent.match(/<Track[^>]*>/g) || [];
    logger.info('Track tags found', {
      count: trackMatches.length,
      matches: trackMatches.slice(0, 3)
    });

    // Try to extract actual release data
    const releases = [];
    const releaseListMatch = xmlContent.match(/<ReleaseList[^>]*>([\s\S]*?)<\/ReleaseList>/);
    logger.info('ReleaseList extraction', {
      found: !!releaseListMatch,
      contentLength: releaseListMatch ? releaseListMatch[1].length : 0,
      first200: releaseListMatch ? releaseListMatch[1].substring(0, 200).replace(/\n/g, ' ').replace(/\s+/g, ' ') : null
    });

    if (releaseListMatch) {
      const releaseContent = releaseListMatch[1];
      const individualReleases = releaseContent.matchAll(/<Release[^>]*>([\s\S]*?)<\/Release>/g);
      
      for (const match of individualReleases) {
        const releaseXml = match[1];
        logger.info('Processing individual release', {
          xmlLength: releaseXml.length,
          first100: releaseXml.substring(0, 100).replace(/\n/g, ' ').replace(/\s+/g, ' ')
        });

        const titleMatch = releaseXml.match(/<TitleText[^>]*>([^<]+)<\/TitleText>/);
        const typeMatch = releaseXml.match(/<ReleaseType[^>]*>([^<]+)<\/ReleaseType>/);
        const refMatch = releaseXml.match(/<ReleaseReference[^>]*>([^<]+)<\/ReleaseReference>/);
        
        const release = {
          releaseId: refMatch ? refMatch[1] : 'NO_ID',
          title: titleMatch ? titleMatch[1] : 'NO_TITLE',
          releaseType: typeMatch ? typeMatch[1] : 'Unknown',
          displayArtist: '',
          trackCount: 0,
          tracks: []
        };

        // Try to get artist
        const artistMatch = releaseXml.match(/<DisplayArtist[^>]*>[\s\S]*?<FullName[^>]*>([^<]+)<\/FullName>/);
        if (artistMatch) {
          release.displayArtist = artistMatch[1];
        }

        logger.info('Extracted release', {
          releaseId: release.releaseId,
          title: release.title,
          releaseType: release.releaseType,
          displayArtist: release.displayArtist
        });
        releases.push(release);
      }
    }

    // Try to extract resources
    const resources = {};
    const resourceListMatch = xmlContent.match(/<ResourceList[^>]*>([\s\S]*?)<\/ResourceList>/);
    logger.info('ResourceList extraction', {
      found: !!resourceListMatch,
      contentLength: resourceListMatch ? resourceListMatch[1].length : 0,
      first200: resourceListMatch ? resourceListMatch[1].substring(0, 200).replace(/\n/g, ' ').replace(/\s+/g, ' ') : null
    });

    if (resourceListMatch) {
      const soundRecordings = resourceListMatch[1].matchAll(/<SoundRecording[^>]*>([\s\S]*?)<\/SoundRecording>/g);
      let resourceCount = 0;
      
      for (const match of soundRecordings) {
        const resourceXml = match[1];
        const refMatch = resourceXml.match(/<ResourceReference[^>]*>([^<]+)<\/ResourceReference>/);
        
        if (refMatch) {
          const resourceId = refMatch[1];
          const titleMatch = resourceXml.match(/<TitleText[^>]*>([^<]+)<\/TitleText>/);
          const isrcMatch = resourceXml.match(/<ISRC[^>]*>([^<]+)<\/ISRC>/);
          
          resources[resourceId] = {
            resourceId: resourceId,
            title: titleMatch ? titleMatch[1] : 'NO_TITLE',
            isrc: isrcMatch ? isrcMatch[1] : ''
          };
          
          resourceCount++;
          logger.info('Extracted resource', {
            resourceId: resourceId,
            title: resources[resourceId].title,
            isrc: resources[resourceId].isrc
          });
        }
      }
      
      logger.info('Total resources extracted', { count: resourceCount });
    }

    // Try to extract deals
    const deals = [];
    const dealListMatch = xmlContent.match(/<DealList[^>]*>([\s\S]*?)<\/DealList>/);
    logger.info('DealList extraction', {
      found: !!dealListMatch,
      contentLength: dealListMatch ? dealListMatch[1].length : 0,
      first200: dealListMatch ? dealListMatch[1].substring(0, 200).replace(/\n/g, ' ').replace(/\s+/g, ' ') : null
    });

    if (dealListMatch) {
      const dealMatches = dealListMatch[1].matchAll(/<ReleaseDeal[^>]*>([\s\S]*?)<\/ReleaseDeal>/g);
      
      for (const match of dealMatches) {
        const dealXml = match[1];
        const refMatch = dealXml.match(/<DealReference[^>]*>([^<]+)<\/DealReference>/);
        const territoryMatch = dealXml.match(/<TerritoryCode[^>]*>([^<]+)<\/TerritoryCode>/);
        
        const deal = {
          dealId: refMatch ? refMatch[1] : 'NO_ID',
          territories: territoryMatch ? [territoryMatch[1]] : [],
          releases: [],
          usageRights: []
        };
        
        logger.info('Extracted deal', {
          dealId: deal.dealId,
          territories: JSON.stringify(deal.territories),
          hasReleases: deal.releases.length
        });
        deals.push(deal);
      }
    }

    const result = {
      messageId: messageIdMatch ? messageIdMatch[1] : 'Unknown',
      messageType: 'NewReleaseMessage',
      messageDate: new Date().toISOString(),
      senderName: 'Cloud Parser',
      senderId: 'CLOUD_PARSER',
      recipientName: 'API Client',
      recipientId: 'API_CLIENT',
      version: 'V4_3',
      releaseCount: releases.length,
      trackCount: trackMatches.length,
      dealCount: deals.length,
      resourceCount: Object.keys(resources).length,
      totalDurationSeconds: 0,
      releases: releases,
      resources: resources,
      deals: deals
    };

    logger.info('SimpleDdexParser.parse COMPLETE', {
      messageId: result.messageId,
      releaseCount: result.releaseCount,
      resourceCount: result.resourceCount,
      dealCount: result.dealCount,
      firstRelease: result.releases.length > 0 ? JSON.stringify(result.releases[0]) : 'none',
      resourceKeys: Object.keys(result.resources).join(','),
      firstDeal: result.deals.length > 0 ? JSON.stringify(result.deals[0]) : 'none'
    });

    return result;
  }
  
  // Add parseSync for compatibility
  parseSync(xmlContent) {
    return this.parse(xmlContent);
  }
}

class SimpleDdexBuilder {
  build(buildRequest) {
    // Simple XML building
    const messageId = buildRequest.messageHeader?.messageId || `msg-${Date.now()}`;
    const sender = buildRequest.messageHeader?.messageSenderName || 'DDEX Suite API';
    const recipient = buildRequest.messageHeader?.messageRecipientName || 'Client';
    const releases = buildRequest.releases || [];

    let xml = `<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/V3_8_2" MessageSchemaVersionId="ern/V3_8_2">
  <MessageHeader>
    <MessageId>${messageId}</MessageId>
    <MessageCreatedDateTime>${new Date().toISOString()}</MessageCreatedDateTime>
    <MessageSender>
      <PartyName>${sender}</PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyName>${recipient}</PartyName>
    </MessageRecipient>
    <MessageThreadId>${messageId}</MessageThreadId>
  </MessageHeader>
  <ResourceList/>
  <ReleaseList>`;

    releases.forEach((release, index) => {
      xml += `
    <Release>
      <ReleaseId>
        <GRid>${release.releaseId || `REL${index + 1}`}</GRid>
      </ReleaseId>
      <ReferenceTitle>
        <TitleText>${release.title || 'Untitled'}</TitleText>
      </ReferenceTitle>
      <ReleaseDetailsByTerritory TerritoryCode="Worldwide">
        <DisplayArtistName>${release.artist || 'Unknown Artist'}</DisplayArtistName>
        <OriginalReleaseDate>${release.releaseDate || '2024-01-01'}</OriginalReleaseDate>
      </ReleaseDetailsByTerritory>
    </Release>`;
    });

    xml += `
  </ReleaseList>
</ern:NewReleaseMessage>`;

    return xml;
  }
  
  // Add buildSync for compatibility
  buildSync(buildRequest) {
    return this.build(buildRequest);
  }
}

// Wrapper class for native parser to handle async/sync properly
class NativeDdexParserWrapper {
  constructor(nativeParser) {
    this.nativeParser = nativeParser;
  }
  
  parse(xmlContent, options) {
    // Use parseSync from native parser
    return this.nativeParser.parseSync(xmlContent, options);
  }
  
  parseSync(xmlContent, options) {
    return this.nativeParser.parseSync(xmlContent, options);
  }
}

class NativeDdexBuilderWrapper {
  constructor(nativeBuilder) {
    this.nativeBuilder = nativeBuilder;
  }
  
  build(buildRequest, options) {
    // Use build from native builder (not buildSync)
    return this.nativeBuilder.build(buildRequest, options);
  }
  
  buildSync(buildRequest, options) {
    // Fallback to build since buildSync doesn't exist
    return this.nativeBuilder.build(buildRequest, options);
  }
}

// Detailed module loading diagnostics
const fs = require('fs');
const path = require('path');

// Read the version of an installed package from its own package.json.
// The ddex packages' "exports" map doesn't expose ./package.json, so resolve the
// entry point and walk up to the package root instead of require('<pkg>/package.json').
function readInstalledVersion(pkgName) {
  try {
    let dir = path.dirname(require.resolve(pkgName));
    while (dir !== path.dirname(dir)) {
      const candidate = path.join(dir, 'package.json');
      if (fs.existsSync(candidate)) {
        const pkg = JSON.parse(fs.readFileSync(candidate, 'utf8'));
        if (pkg.name === pkgName) return pkg.version;
      }
      dir = path.dirname(dir);
    }
  } catch (err) {
    logger.warn(`Could not determine installed version of ${pkgName}: ${err.message}`);
  }
  return null;
}

const PACKAGE_VERSIONS = {
  parser: readInstalledVersion('ddex-parser'),
  builder: readInstalledVersion('ddex-builder'),
};

try {
  logger.info("=== NATIVE MODULE LOADING DIAGNOSTICS ===");
  logger.info("Environment:", {
    platform: process.platform,
    arch: process.arch,
    nodeVersion: process.version,
    cwd: process.cwd(),
    dirname: __dirname,
    env_NODE_ENV: process.env.NODE_ENV
  });

  // Check if ddex-parser directory exists
  const ddexParserPath = path.join(__dirname, 'node_modules', 'ddex-parser');
  logger.info("Checking ddex-parser path:", {
    path: ddexParserPath,
    exists: fs.existsSync(ddexParserPath)
  });

  if (fs.existsSync(ddexParserPath)) {
    const files = fs.readdirSync(ddexParserPath);
    logger.info("Files in ddex-parser:", {
      count: files.length,
      files: files.filter(f => f.endsWith('.node') || f.endsWith('.js'))
    });

    // Check for specific binary files
    const binaries = [
      'ddex-parser-node.linux-x64-gnu.node',
      'ddex-parser.linux-x64-gnu.node',
      'index.linux-x64-gnu.node'
    ];

    binaries.forEach(binary => {
      const binaryPath = path.join(ddexParserPath, binary);
      if (fs.existsSync(binaryPath)) {
        const stats = fs.statSync(binaryPath);
        logger.info(`Binary ${binary} found:`, {
          size: stats.size,
          mode: stats.mode.toString(8),
          path: binaryPath
        });
      }
    });

    // Check the main index.js
    const indexPath = path.join(ddexParserPath, 'index.js');
    if (fs.existsSync(indexPath)) {
      const indexContent = fs.readFileSync(indexPath, 'utf8');
      logger.info("ddex-parser index.js preview:", {
        length: indexContent.length,
        first200: indexContent.substring(0, 200)
      });
    }
  }

  logger.info("Attempting require('ddex-parser')...");
  const ddexParserModule = require("ddex-parser");

  logger.info("ddex-parser loaded successfully:", {
    moduleType: typeof ddexParserModule,
    exports: Object.keys(ddexParserModule),
    hasDdexParser: 'DdexParser' in ddexParserModule
  });

  const ddexBuilderModule = require("ddex-builder");
  logger.info("ddex-builder loaded successfully:", {
    exports: Object.keys(ddexBuilderModule)
  });

  // Try to instantiate
  const NativeDdexParser = ddexParserModule.DdexParser;
  const NativeDdexBuilder = ddexBuilderModule.DdexBuilder;

  logger.info("Creating test parser instance...");
  const testParser = new NativeDdexParser();
  const testBuilder = new NativeDdexBuilder();

  logger.info("Parser instance created:", {
    hasParseSync: typeof testParser.parseSync === 'function',
    hasParse: typeof testParser.parse === 'function',
    methods: Object.getOwnPropertyNames(Object.getPrototypeOf(testParser))
  });

  logger.info("Builder instance created:", {
    hasBuildSync: typeof testBuilder.buildSync === 'function',
    hasBuild: typeof testBuilder.build === 'function',
    methods: testBuilder.build ? Object.getOwnPropertyNames(Object.getPrototypeOf(testBuilder)) : []
  });

  // Check that the required methods exist - parser needs parseSync, builder needs build
  if (typeof testParser.parseSync === 'function' && typeof testBuilder.build === 'function') {
    logger.info("Native modules validation successful - required methods exist");
    
    // Native modules work!
    DdexParser = NativeDdexParserWrapper;
    DdexBuilder = NativeDdexBuilderWrapper;
    nativeModulesAvailable = true;
    parserType = 'native';

    // Create factory functions
    const originalDdexParser = DdexParser;
    const originalDdexBuilder = DdexBuilder;

    DdexParser = class {
      constructor() {
        return new originalDdexParser(new NativeDdexParser());
      }
    };

    DdexBuilder = class {
      constructor() {
        return new originalDdexBuilder(new NativeDdexBuilder());
      }
    };

    logger.info("=== NATIVE MODULES LOADED SUCCESSFULLY ===");
  } else {
    throw new Error("Required methods not found on native modules");
  }

} catch (error) {
  logger.error("=== NATIVE MODULE LOAD FAILED ===", {
    errorMessage: error.message,
    errorCode: error.code,
    errorStack: error.stack?.split('\n').slice(0, 5),
    requirePaths: module.paths
  });

  // Try to get more specific error info
  if (error.message.includes('Cannot find module')) {
    logger.error("Module not found - checking require.resolve:", {
      canResolve: false
    });
  } else if (error.message.includes('.node')) {
    logger.error("Binary loading error - likely incompatible binary");
  }

  // Fallback to SimpleDdexParser
  DdexParser = SimpleDdexParser;
  DdexBuilder = SimpleDdexBuilder;
  nativeModulesAvailable = false;
  parserType = 'fallback';

  logger.info("Using fallback SimpleDdexParser");
}

// Set global options for cost control
setGlobalOptions({ maxInstances: 10 });

// Helper function to handle CORS
function setCORSHeaders(res) {
  res.set('Access-Control-Allow-Origin', '*');
  res.set('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.set('Access-Control-Allow-Headers', 'Content-Type, Authorization');
}

// Helper function to handle errors
function handleError(res, error, operation) {
  logger.error(`Error in ${operation}:`, error);
  res.status(500).json({
    success: false,
    error: error.message || 'Internal server error',
    operation,
    parser: parserType
  });
}

// Parser API endpoint
exports.parseXML = onRequest({
  memory: '1GiB',
  timeoutSeconds: 60,
  maxInstances: 5,
  cors: true
}, async (req, res) => {
  setCORSHeaders(res);

  if (req.method === 'OPTIONS') {
    res.status(200).send();
    return;
  }

  if (req.method !== 'POST') {
    res.status(405).json({
      success: false,
      error: 'Method not allowed. Use POST.'
    });
    return;
  }

  try {
    let xmlContent = req.body.xmlContent || req.body;
    const options = req.body.options || {};

    // Handle different content types
    if (typeof xmlContent !== 'string') {
      if (Buffer.isBuffer(xmlContent)) {
        xmlContent = xmlContent.toString('utf-8');
      } else if (typeof xmlContent === 'object' && xmlContent.xml) {
        xmlContent = xmlContent.xml;
      } else {
        xmlContent = JSON.stringify(xmlContent);
      }
    }

    if (!xmlContent) {
      res.status(400).json({
        success: false,
        error: 'Missing xmlContent in request body'
      });
      return;
    }

    logger.info('Parsing DDEX XML', {
      contentLength: xmlContent.length,
      options,
      parser: parserType
    });

    const startTime = Date.now();
    const parser = new DdexParser();
    const result = parser.parse(xmlContent, options);
    const parseTime = Date.now() - startTime;

    logger.info('Successfully parsed DDEX XML', {
      messageId: result.messageId,
      releaseCount: result.releaseCount || 0,
      parseTimeMs: parseTime,
      parser: parserType
    });

    res.json({
      success: true,
      data: result,
      metadata: {
        contentLength: xmlContent.length,
        parseTime: new Date().toISOString(),
        parseTimeMs: parseTime,
        parser: parserType
      }
    });

  } catch (error) {
    handleError(res, error, 'parseXML');
  }
});

// Builder API endpoint
exports.buildXML = onRequest({
  memory: '1GiB',
  timeoutSeconds: 60,
  maxInstances: 5,
  cors: true
}, async (req, res) => {
  setCORSHeaders(res);

  if (req.method === 'OPTIONS') {
    res.status(200).send();
    return;
  }

  if (req.method !== 'POST') {
    res.status(405).json({
      success: false,
      error: 'Method not allowed. Use POST.'
    });
    return;
  }

  try {
    const { buildRequest, options = {} } = req.body;

    if (!buildRequest) {
      res.status(400).json({
        success: false,
        error: 'Missing buildRequest in request body'
      });
      return;
    }

    logger.info('Building DDEX XML', {
      hasFlat: !!buildRequest.flat,
      hasGraph: !!buildRequest.graph,
      options,
      builder: parserType
    });

    const startTime = Date.now();
    const builder = new DdexBuilder();
    const xmlResult = builder.build(buildRequest, options);
    const buildTime = Date.now() - startTime;

    logger.info('Successfully built DDEX XML', {
      outputLength: xmlResult.length,
      buildTimeMs: buildTime,
      builder: parserType
    });

    res.json({
      success: true,
      data: {
        xml: xmlResult
      },
      metadata: {
        outputLength: xmlResult.length,
        buildTime: new Date().toISOString(),
        buildTimeMs: buildTime,
        builder: parserType
      }
    });

  } catch (error) {
    handleError(res, error, 'buildXML');
  }
});

// Round-trip API endpoint (Parse → Modify → Build)
exports.roundTrip = onRequest({
  memory: '1GiB',
  timeoutSeconds: 120,
  maxInstances: 3,
  cors: true
}, async (req, res) => {
  setCORSHeaders(res);

  if (req.method === 'OPTIONS') {
    res.status(200).send();
    return;
  }

  if (req.method !== 'POST') {
    res.status(405).json({
      success: false,
      error: 'Method not allowed. Use POST.'
    });
    return;
  }

  try {
    let xmlContent = req.body.xmlContent || req.body;
    const modifications = req.body.modifications || {};
    const options = req.body.options || {};

    // Handle different content types
    if (typeof xmlContent !== 'string') {
      if (Buffer.isBuffer(xmlContent)) {
        xmlContent = xmlContent.toString('utf-8');
      } else if (typeof xmlContent === 'object' && xmlContent.xml) {
        xmlContent = xmlContent.xml;
      }
    }

    if (!xmlContent) {
      res.status(400).json({
        success: false,
        error: 'Missing xmlContent in request body'
      });
      return;
    }

    logger.info('Starting round-trip operation', {
      contentLength: xmlContent.length,
      hasModifications: Object.keys(modifications).length > 0,
      parser: parserType
    });

    const startTime = Date.now();

    // Step 1: Parse the XML
    const parser = new DdexParser();
    const parsed = parser.parse(xmlContent, options.parser);

    // Step 2: Apply modifications if provided
    let modified = parsed;
    if (Object.keys(modifications).length > 0) {
      modified = { ...parsed };

      // Apply modifications to flat representation
      if (modifications.flat && modified.flat) {
        Object.assign(modified.flat, modifications.flat);
      }

      // Apply modifications to graph representation
      if (modifications.graph && modified.graph) {
        Object.assign(modified.graph, modifications.graph);
      }
    }

    // Step 3: Build the XML back
    const builder = new DdexBuilder();
    const buildRequest = modified.toBuildRequest ? modified.toBuildRequest() : modified;
    const xmlResult = builder.build(buildRequest, options.builder);
    
    const totalTime = Date.now() - startTime;

    logger.info('Successfully completed round-trip', {
      originalLength: xmlContent.length,
      outputLength: xmlResult.length,
      totalTimeMs: totalTime,
      parser: parserType
    });

    res.json({
      success: true,
      data: {
        original: parsed,
        modified: modified,
        xml: xmlResult
      },
      metadata: {
        originalLength: xmlContent.length,
        outputLength: xmlResult.length,
        roundTripTime: new Date().toISOString(),
        totalTimeMs: totalTime,
        parser: parserType
      }
    });

  } catch (error) {
    handleError(res, error, 'roundTrip');
  }
});

// Health check endpoint
exports.health = onRequest({
  memory: '256MiB',
  timeoutSeconds: 10,
  cors: true
}, (req, res) => {
  setCORSHeaders(res);

  res.json({
    success: true,
    message: 'DDEX Suite API is running',
    timestamp: new Date().toISOString(),
    version: PACKAGE_VERSIONS.parser,
    packageVersions: PACKAGE_VERSIONS,
    modules: {
      parser: !!DdexParser,
      builder: !!DdexBuilder,
      native: nativeModulesAvailable,
      type: parserType
    }
  });
});

// API documentation endpoint
exports.docs = onRequest({
  memory: '256MiB',
  timeoutSeconds: 10,
  cors: true
}, (req, res) => {
  setCORSHeaders(res);

  const docs = {
    title: 'DDEX Suite API',
    version: PACKAGE_VERSIONS.parser,
    packageVersions: PACKAGE_VERSIONS,
    description: 'REST API for DDEX XML parsing and building',
    parser: parserType,
    native: nativeModulesAvailable,
    endpoints: {
      '/parseXML': {
        method: 'POST',
        description: 'Parse DDEX XML into structured data',
        body: {
          xmlContent: 'string (required) - The DDEX XML content to parse',
          options: 'object (optional) - Parser options'
        },
        response: {
          success: 'boolean',
          data: 'object - Parsed DDEX data with flat and graph representations',
          metadata: 'object - Parse metadata including parser type'
        }
      },
      '/buildXML': {
        method: 'POST',
        description: 'Build DDEX XML from structured data',
        body: {
          buildRequest: 'object (required) - The build request with flat/graph data',
          options: 'object (optional) - Builder options'
        },
        response: {
          success: 'boolean',
          data: 'object - Generated XML content',
          metadata: 'object - Build metadata'
        }
      },
      '/roundTrip': {
        method: 'POST',
        description: 'Parse → Modify → Build workflow',
        body: {
          xmlContent: 'string (required) - The DDEX XML content to process',
          modifications: 'object (optional) - Modifications to apply',
          options: 'object (optional) - Parser and builder options'
        },
        response: {
          success: 'boolean',
          data: 'object - Original, modified, and rebuilt data',
          metadata: 'object - Round-trip metadata'
        }
      },
      '/health': {
        method: 'GET',
        description: 'API health check',
        response: {
          success: 'boolean',
          message: 'string',
          timestamp: 'string',
          version: 'string',
          modules: 'object - Module status'
        }
      }
    }
  };

  res.json(docs);
});

// Enhanced DDEX parse handler for playground with extensive logging
const ddexParseHandler = async (req, res) => {
  try {
    // Create parser with debug disabled for native modules
    let parser;
    if (nativeModulesAvailable) {
      // For native parser, use DdexParser with debug: false
      const { DdexParser: NativeDdexParser } = require('ddex-parser');
      parser = new NativeDdexParser({ debug: false });
    } else {
      // For fallback parser, use existing wrapper
      parser = new DdexParser();
    }

    const xml = req.body.xml || req.body;

    // Use parseSync to avoid debug output
    let result = parser.parseSync(xml);

    // Workaround for v0.4.3 bug: manually extract release titles
    if (result.releases && result.releases.length > 0) {
      // Extract all TitleText elements for releases
      const releaseTitles = [];
      const titleRegex = /<Release[^>]*>[\s\S]*?<TitleText>([^<]+)<\/TitleText>/g;
      let match;
      while ((match = titleRegex.exec(xml)) !== null) {
        releaseTitles.push(match[1]);
      }

      // Extract artist names for releases
      const artistRegex = /<DisplayArtist[^>]*>[\s\S]*?<FullName>([^<]+)<\/FullName>/g;
      const artists = [];
      while ((match = artistRegex.exec(xml)) !== null) {
        artists.push(match[1]);
      }

      // Apply the extracted values
      result.releases.forEach((release, index) => {
        if (releaseTitles[index]) {
          release.title = releaseTitles[index];
          release.defaultTitle = releaseTitles[index];
        }
        if (artists[index]) {
          release.displayArtist = artists[index];
        }
      });

      // Also extract resources if present
      if (xml.includes('<SoundRecording>')) {
        const trackTitles = [];
        const trackRegex = /<SoundRecording[^>]*>[\s\S]*?<TitleText>([^<]+)<\/TitleText>/g;
        while ((match = trackRegex.exec(xml)) !== null) {
          trackTitles.push(match[1]);
        }

        // Apply to tracks if they exist
        if (result.tracks && trackTitles.length > 0) {
          result.tracks.forEach((track, index) => {
            if (trackTitles[index]) {
              track.title = trackTitles[index];
            }
          });
        }
      }
    }

    console.log('Parse complete with workaround applied');
    console.log('MessageId:', result.messageId);
    console.log('First release:', result.releases?.[0]?.title);

    res.json(result);
  } catch (error) {
    console.error('Parse error:', error.message);
    res.status(500).json({ error: error.message });
  }
};

// Enhanced DDEX build handler for playground with comprehensive logging
const ddexBuildHandler = async (req, res) => {
  try {
    const builder = new DdexBuilder();
    const inputData = req.body;

    console.log('Building DDEX from data with messageId:', inputData.messageId);

    // build() returns a Promise that resolves to the XML string directly!
    const xmlString = await builder.build(inputData);

    console.log('Build complete, XML length:', xmlString.length);

    res.json({ xml: xmlString });
  } catch (error) {
    console.error('Build error:', error.message);
    res.status(500).json({ error: error.message });
  }
};

// DDEX API endpoints using custom handlers
exports.ddexParse = onRequest({
  memory: '1GiB',
  timeoutSeconds: 60,
  cors: true
}, ddexParseHandler);

exports.ddexBuild = onRequest({
  memory: '1GiB',
  timeoutSeconds: 60,
  cors: true
}, ddexBuildHandler);

exports.ddexBatch = onRequest({
  memory: '1GiB',
  timeoutSeconds: 60,
  cors: true
}, batchHandler);