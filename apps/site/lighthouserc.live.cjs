/* global module, require, structuredClone */
// LHCI loads this CommonJS configuration through require.
// eslint-disable-next-line @typescript-eslint/no-require-imports
const config = structuredClone(require('./lighthouserc.cjs'));
delete config.ci.collect.startServerCommand;
delete config.ci.collect.startServerReadyPattern;
delete config.ci.collect.startServerReadyTimeout;
config.ci.collect.url = config.ci.collect.url.map((url) =>
  url.replace('http://127.0.0.1:4323', 'https://tanzir71.github.io'),
);
config.ci.upload.outputDir = '../../artifacts/p6/lighthouse/live';
module.exports = config;
