// SPDX-License-Identifier: Apache-2.0
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  testMatch: 'production-accessibility.spec.mjs',
  fullyParallel: false,
  workers: 1,
  timeout: 120_000,
  outputDir: '.next/playwright-results',
  reporter: 'list',
  use: {
    baseURL: 'http://127.0.0.1:4174',
    browserName: 'chromium',
    headless: true,
  },
  webServer: {
    command: 'node scripts/serve-export.mjs out 4174',
    url: 'http://127.0.0.1:4174/',
    reuseExistingServer: false,
    timeout: 10_000,
  },
});
