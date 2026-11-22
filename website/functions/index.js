/**
 * DDEX Suite Playground API (Cloud Functions for Firebase, 2nd gen).
 *
 *   POST /api/parse   {"xml": "<ern:NewReleaseMessage ...>"}  -> the parser's ParsedMessage
 *   POST /api/build   <a ddex-builder BuildRequest>          -> {"xml": "..."}
 *   GET  /api/health                                         -> {status, ddexParser, ddexBuilder}
 *
 * Errors are {"error": "<message>", "code": "<CODE>"}. They never contain the request
 * content or raw parser/builder internals; details are logged server-side (without input).
 */

const fs = require('fs');
const path = require('path');
const { onRequest } = require('firebase-functions/v2/https');
const logger = require('firebase-functions/logger');

// Request bodies above this size are rejected with 413 (the Playground checks the same limit).
const MAX_BODY_BYTES = 5 * 1024 * 1024;

const ALLOWED_ORIGINS = [
  'https://ddex-suite.org',
  'https://ddex-suite.web.app',
  'http://localhost:3000',
];

// Sizing (see the PR description): parsing a 5 MB message peaks at about 90 MB above the
// Node baseline, so 512 MiB leaves room for 4 concurrent requests on one vCPU.
const API_OPTIONS = {
  memory: '512MiB',
  cpu: 1,
  concurrency: 4,
  maxInstances: 5,
  timeoutSeconds: 30,
  cors: ALLOWED_ORIGINS,
};

// Read an installed package's version from its own package.json. The ddex packages'
// "exports" map doesn't expose ./package.json, so walk up from the resolved entry point.
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
    // Reported through the null version below.
  }
  return null;
}

const VERSIONS = {
  ddexParser: readInstalledVersion('ddex-parser'),
  ddexBuilder: readInstalledVersion('ddex-builder'),
};

let DdexParser = null;
let DdexBuilder = null;
try {
  ({ DdexParser } = require('ddex-parser'));
  ({ DdexBuilder } = require('ddex-builder'));
  logger.info('ddex native modules loaded', VERSIONS);
} catch (err) {
  DdexParser = null;
  DdexBuilder = null;
  logger.error('ddex native modules failed to load', { message: err.message, code: err.code });
}

function sendError(res, status, code, message) {
  res.status(status).json({ error: message, code });
}

// Common checks for the POST endpoints. Returns false when a response was already sent.
function checkPost(req, res) {
  if (req.method !== 'POST') {
    res.set('Allow', 'POST');
    sendError(res, 405, 'METHOD_NOT_ALLOWED', 'Use POST.');
    return false;
  }
  const declared = Number(req.get('content-length') || 0);
  const actual = req.rawBody ? req.rawBody.length : 0;
  if (declared > MAX_BODY_BYTES || actual > MAX_BODY_BYTES) {
    sendError(res, 413, 'PAYLOAD_TOO_LARGE', 'The request body is larger than the 5 MB limit.');
    return false;
  }
  if (!DdexParser || !DdexBuilder) {
    sendError(res, 503, 'SERVICE_UNAVAILABLE', 'The service is temporarily unavailable.');
    return false;
  }
  if (!req.is('application/json') || req.body === null || typeof req.body !== 'object' || Array.isArray(req.body)) {
    sendError(res, 400, 'INVALID_REQUEST', 'Send a JSON object with Content-Type: application/json.');
    return false;
  }
  return true;
}

// The builder's validation messages name the missing field (e.g. "ERN 4.3 requires
// Release/ReleaseType (releases[0].release_type)") and are safe to return. Anything else
// is replaced by a generic message.
function publicBuildError(message) {
  const requires = /ERN [\d.]+ requires [^\n]*?\([^)\n]*\)/.exec(message);
  if (requires) return requires[0];
  const missing = /missing field `([A-Za-z0-9_]+)`/.exec(message);
  if (missing) return `The build request is missing the field \`${missing[1]}\`.`;
  return null;
}

exports.ddexParse = onRequest(API_OPTIONS, async (req, res) => {
  if (!checkPost(req, res)) return;

  const { xml } = req.body;
  if (typeof xml !== 'string' || xml.trim() === '') {
    sendError(res, 400, 'INVALID_REQUEST', 'The request needs an "xml" string.');
    return;
  }

  try {
    const result = await new DdexParser().parse(xml);
    res.json(result);
  } catch (err) {
    // The parser's message can quote the input, so only its length is logged.
    logger.warn('parse failed', { inputBytes: Buffer.byteLength(xml), kind: err.code || 'ParseError' });
    sendError(
      res,
      422,
      'PARSE_FAILED',
      'The XML could not be parsed as a DDEX ERN 3.8.2, 4.2 or 4.3 message.',
    );
  }
});

exports.ddexBuild = onRequest(API_OPTIONS, async (req, res) => {
  if (!checkPost(req, res)) return;

  try {
    const xml = await new DdexBuilder().build(req.body);
    res.json({ xml });
  } catch (err) {
    const message = publicBuildError(String(err.message || ''));
    if (message) {
      sendError(res, 422, 'INVALID_BUILD_REQUEST', message);
      return;
    }
    logger.warn('build failed', { kind: err.code || 'BuildError' });
    sendError(
      res,
      422,
      'BUILD_FAILED',
      'The build request is not valid. Check it against the ddex-builder BuildRequest type.',
    );
  }
});

exports.health = onRequest(
  { memory: '256MiB', cpu: 'gcf_gen1', maxInstances: 2, timeoutSeconds: 10, cors: ALLOWED_ORIGINS },
  (req, res) => {
    if (req.method !== 'GET') {
      res.set('Allow', 'GET');
      sendError(res, 405, 'METHOD_NOT_ALLOWED', 'Use GET.');
      return;
    }
    const ok = Boolean(DdexParser && DdexBuilder);
    res.set('Cache-Control', 'no-store');
    res.status(ok ? 200 : 503).json({
      status: ok ? 'ok' : 'unavailable',
      ddexParser: VERSIONS.ddexParser,
      ddexBuilder: VERSIONS.ddexBuilder,
    });
  },
);
