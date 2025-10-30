import type { NextConfig } from "next";

// XXX we don't support Turbopack due to
// https://github.com/vercel/next.js/issues/85119
// https://github.com/vercel/next.js/discussions/77102
// https://github.com/emscripten-core/emscripten/issues/20580
// (for a more similar description to our problem)

const nextConfig: NextConfig = {
  headers: async () => [{
    source: '/:path*',
    headers: [
      {
        key: 'Cross-Origin-Embedder-Policy',
        value: 'require-corp',
      },
      {
        key: 'Cross-Origin-Opener-Policy',
        value: 'same-origin',
      },
    ],
  }],
};

export default nextConfig;
