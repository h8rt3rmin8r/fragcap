// SPDX-License-Identifier: Apache-2.0
import { createMDX } from 'fumadocs-mdx/next';

const withMDX = createMDX();

/**
 * Static export, configured once (specification section 22.2):
 * - output export: a fully static site, no server.
 * - images unoptimized: there is no server to optimize them.
 * - no basePath: served from the domain root at fragcap.com, not a subpath.
 * The .nojekyll marker and CNAME are written into the export root by
 * scripts/postbuild.mjs after the build.
 *
 * @type {import('next').NextConfig}
 */
const config = {
  output: 'export',
  images: { unoptimized: true },
  reactStrictMode: true,
};

export default withMDX(config);
