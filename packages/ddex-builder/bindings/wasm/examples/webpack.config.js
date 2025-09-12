const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports = {
  entry: './src/index.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'index.js',
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: path.resolve(__dirname, 'public/index.html')
    }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, '..'),
      outDir: path.resolve(__dirname, '../pkg'),
      outName: 'ddex_builder_wasm',
      forceMode: 'production'
    }),
  ],
  mode: 'production',
  experiments: {
    asyncWebAssembly: true,
  },
  performance: {
    maxEntrypointSize: 600000,  // Allow larger WASM bundles
    maxAssetSize: 600000,
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        wasm: {
          test: /\.wasm$/,
          name: 'wasm-modules',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
};