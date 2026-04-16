// Native Node.js bindings (.node files compiled from Rust) cannot run in
// browser environments — they are binary extensions that require the Node.js
// runtime and OS-level dynamic linking. Bundlers (Webpack, Vite, etc.) that
// encounter this package in a browser target will load this shim instead,
// producing a clear error rather than a cryptic binary load failure.
throw new Error(
  '@quicknode/sdk-next does not support browser environments. ' +
  'This package requires Node.js.'
);
