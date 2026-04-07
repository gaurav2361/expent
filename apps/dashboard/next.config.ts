import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  /* config options here */
  output: "standalone",
  reactCompiler: true,
  experimental: {
    viewTransition: true,
  },
};

export default nextConfig;
