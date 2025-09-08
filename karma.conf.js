// karma.conf.js
module.exports = function(config) {
  config.set({
    frameworks: ['mocha', 'chai'],
    files: [
      'dist/browser.js',
      'test/browser/**/*.test.js'
    ],
    browsers: ['Chrome', 'Firefox', 'Safari', 'Edge'],
    customLaunchers: {
      ChromeHeadless: {
        base: 'Chrome',
        flags: ['--headless', '--disable-gpu']
      }
    },
    singleRun: true,
    reporters: ['progress', 'coverage']
  });
};