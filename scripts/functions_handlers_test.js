#!/usr/bin/env node
// Calls the Playground API handlers in website/functions/index.js (health, ddexParse,
// ddexBuild) directly, with minimal Express-like request / response objects, using the
// installed ddex-parser / ddex-builder packages (run `npm ci` in website/functions first).
// No emulator or deploy: this checks the handlers' own logic and that the native modules
// load and work with the locked versions.
//
// Usage: node scripts/functions_handlers_test.js website/functions "examples/Samples43/1 Audio.xml"

const assert = require('node:assert/strict');
const { EventEmitter } = require('node:events');
const fs = require('node:fs');
const path = require('node:path');

const functionsDir = path.resolve(process.argv[2] || 'website/functions');
const samplePath = process.argv[3] || 'examples/Samples43/1 Audio.xml';
const api = require(path.join(functionsDir, 'index.js'));
const expectedVersion = (name) =>
  JSON.parse(fs.readFileSync(path.join(functionsDir, 'package.json'), 'utf8')).dependencies[name].replace(/^[\^~=]/, '');

function request(method, body) {
  const raw = body === undefined ? undefined : Buffer.from(JSON.stringify(body));
  const headers = {
    origin: 'https://ddex-suite.org',
    ...(raw ? { 'content-type': 'application/json', 'content-length': String(raw.length) } : {}),
  };
  return {
    method,
    headers,
    body,
    rawBody: raw,
    url: '/',
    get: (name) => headers[name.toLowerCase()],
    header: (name) => headers[name.toLowerCase()],
    is: (type) => (headers['content-type'] || '').includes(type.replace('application/', '')),
  };
}

function response() {
  const res = new EventEmitter();
  res.statusCode = 200;
  res.headers = {};
  res.body = undefined;
  res.setHeader = (k, v) => { res.headers[k.toLowerCase()] = v; };
  res.getHeader = (k) => res.headers[k.toLowerCase()];
  res.removeHeader = (k) => { delete res.headers[k.toLowerCase()]; };
  res.set = (k, v) => { res.setHeader(k, v); return res; };
  res.status = (code) => { res.statusCode = code; return res; };
  res.json = (obj) => { res.body = obj; res.emit('finish'); return res; };
  res.send = (data) => { res.body = data; res.emit('finish'); return res; };
  res.end = (data) => { if (data !== undefined) res.body = data; res.emit('finish'); return res; };
  return res;
}

async function call(handler, method, body) {
  const req = request(method, body);
  const res = response();
  await handler(req, res);
  return res;
}

const party = (name, id) => ({ party_name: [{ text: name }], party_id: id });
const BUILD_REQUEST = {
  header: {
    message_id: 'HANDLER-TEST',
    message_sender: party('Test Label', 'PADPIDA0000000001'),
    message_recipient: party('Test DSP', 'PADPIDA0000000002'),
    message_created_date_time: '2024-09-13T12:00:00Z',
  },
  version: '4.3',
  avs_version_id: '12',
  language_and_script_code: 'en',
  releases: [{
    release_id: 'A1-ABCDE-0000000001-M',
    title: [{ text: 'Test Single' }],
    artist: 'Test Artist',
    label: 'Test Label',
    release_type: 'Single',
    genre: 'Pop',
    parental_warning_type: 'NotExplicit',
    tracks: [{
      track_id: 'T1',
      isrc: 'USRT12400001',
      title: 'Test Track',
      duration: 'PT3M42S',
      artist: 'Test Artist',
      resource_type: 'MusicalWorkSoundRecording',
      parental_warning_type: 'NotExplicit',
    }],
  }],
  deals: [],
};

const tests = [
  ['health: 200 ok with the locked package versions', async () => {
    const res = await call(api.health, 'GET');
    assert.equal(res.statusCode, 200);
    assert.deepEqual(res.body, {
      status: 'ok',
      ddexParser: expectedVersion('ddex-parser'),
      ddexBuilder: expectedVersion('ddex-builder'),
    });
    assert.equal(res.headers['access-control-allow-origin'], 'https://ddex-suite.org');
  }],
  ['health: POST is 405', async () => {
    const res = await call(api.health, 'POST', {});
    assert.equal(res.statusCode, 405);
    assert.equal(res.body.code, 'METHOD_NOT_ALLOWED');
  }],
  ['parse: an official ERN 4.3 sample', async () => {
    const res = await call(api.ddexParse, 'POST', { xml: fs.readFileSync(samplePath, 'utf8') });
    assert.equal(res.statusCode, 200, JSON.stringify(res.body));
    assert.ok(res.body.messageId, 'no messageId');
    assert.ok(res.body.releaseCount >= 1, 'no release');
  }],
  ['parse: malformed XML is 422 PARSE_FAILED, without the input', async () => {
    const res = await call(api.ddexParse, 'POST', { xml: '<ern:NewReleaseMessage>&secret;' });
    assert.equal(res.statusCode, 422);
    assert.equal(res.body.code, 'PARSE_FAILED');
    assert.ok(!JSON.stringify(res.body).includes('secret'));
  }],
  ['parse: missing xml is 400', async () => {
    const res = await call(api.ddexParse, 'POST', {});
    assert.equal(res.statusCode, 400);
    assert.equal(res.body.code, 'INVALID_REQUEST');
  }],
  ['build: a complete ERN 4.3 request', async () => {
    const res = await call(api.ddexBuild, 'POST', BUILD_REQUEST);
    assert.equal(res.statusCode, 200, JSON.stringify(res.body));
    assert.match(res.body.xml, /<ern:NewReleaseMessage/);
    assert.match(res.body.xml, /HANDLER-TEST/);
  }],
  ['build: an incomplete request is 422 and names the missing field', async () => {
    const { release_type, ...release } = BUILD_REQUEST.releases[0];
    const res = await call(api.ddexBuild, 'POST', { ...BUILD_REQUEST, releases: [release] });
    assert.equal(res.statusCode, 422);
    assert.equal(res.body.code, 'INVALID_BUILD_REQUEST');
    assert.match(res.body.error, /release_type/);
  }],
];

(async () => {
  let failed = 0;
  for (const [name, fn] of tests) {
    try {
      await fn();
      console.log(`ok   ${name}`);
    } catch (err) {
      failed += 1;
      console.log(`FAIL ${name}\n     ${err.message}`);
    }
  }
  console.log(failed ? `${failed} of ${tests.length} failed` : `all ${tests.length} handler tests passed`);
  process.exit(failed ? 1 : 0);
})();
