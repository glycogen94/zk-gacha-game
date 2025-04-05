/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  transpilePackages: ['zk-circuits'],
  webpack(config, { isServer }) {
    // Enable WebAssembly
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
      layers: true,
    };

    // Prevent WASM from being processed server-side (unless specifically intended)
    if (isServer) {
      config.externals.push(/zk-circuits/);
    }

    // Organize WASM output files
    config.output.webassemblyModuleFilename = 'static/wasm/[modulehash].wasm';

    return config;
  },
};

export default nextConfig;
