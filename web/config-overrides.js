const path = require('path');

const { removeModuleScopePlugin } = require('customize-cra');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = function override(config, env) {
  config.plugins.push(
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "../"),
      withTypeScript: true,
      outName: 'cidr_aggregator',
      extraArgs: '--features wasm',
    }));

  // Enable async WASM support for webpack 5
  config.experiments = {
    ...config.experiments,
    asyncWebAssembly: true,
  };

  // Fix fullySpecified issue with MUI v9 and react-transition-group
  config.module.rules.push({
    test: /\.m?js$/,
    resolve: {
      fullySpecified: false,
    },
  });

  // In CRA's webpack 5 oneOf rules, the catch-all asset/resource rule at the
  // end does NOT exclude .wasm files. We need to add .wasm to its exclude list
  // so webpack 5's asyncWebAssembly experiment can handle them natively with
  // proper named exports (required by wasm-bindgen).
  config.module.rules.forEach(rule => {
    (rule.oneOf || []).forEach(oneOf => {
      // Handle the catch-all asset/resource rule (last in oneOf)
      if (oneOf.type === 'asset/resource' && oneOf.exclude) {
        const prevExclude = Array.isArray(oneOf.exclude) ? oneOf.exclude : [oneOf.exclude];
        oneOf.exclude = [...prevExclude, /\.wasm$/];
      }
      // Also handle file-loader rules from older configs
      if (oneOf.loader && oneOf.loader.indexOf('file-loader') >= 0) {
        const prevExclude = oneOf.exclude;
        oneOf.exclude = [
          ...(Array.isArray(prevExclude) ? prevExclude : [prevExclude].filter(Boolean)),
          /\.wasm$/,
        ];
      }
    });
  });

  removeModuleScopePlugin()(config);

  return config;
}

// Ref:
//  https://github.com/rustwasm/rust-webpack-template/issues/43#issuecomment-426597176
//  https://prestonrichey.com/blog/react-rust-wasm/
