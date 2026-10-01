import { defineConfig, devices } from '@playwright/test';
export default defineConfig({
  testDir: '.', testMatch: '*.spec.js', timeout: 60000,
  workers: 1, retries: 0,
  outputDir: '../../target/browser-results',
  reporter: [['list'], ['html', { outputFolder: '../../target/browser-report', open: 'never' }]],
  use: { trace: 'retain-on-failure', screenshot: 'only-on-failure' },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'], launchOptions: { args: ['--no-sandbox','--disable-dev-shm-usage'] } } },
    // Playwright 1.63 (Firefox 155) can lose the navigation-committed message when Firefox swaps
    // the browsing context for a Cross-Origin-Opener-Policy: same-origin document (microsoft/playwright#42731,
    // fixed after 1.63). The page loads, but page.goto never resolves under any waitUntil. The portal sends
    // COOP: same-origin on its pages, so a navigation from one to a page without it (for example /apps then the
    // 409 /setup page) or the first navigation to a portal page can hang until the test timeout. This turns off
    // only Firefox's COOP browsing-context swap in the test browser. The server still sends the header, and
    // tests/portal.rs asserts it. Remove this once the Playwright dependency is 1.64 or later.
    { name: 'firefox', use: { ...devices['Desktop Firefox'], launchOptions: { firefoxUserPrefs: { 'browser.tabs.remote.useCrossOriginOpenerPolicy': false } } } },
    { name: 'webkit', use: { ...devices['Desktop Safari'] } }
  ]
});
