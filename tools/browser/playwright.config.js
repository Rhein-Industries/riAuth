import { defineConfig, devices } from '@playwright/test';
export default defineConfig({
  testDir: '.', testMatch: '*.spec.js', timeout: 60000,
  workers: 1, retries: 0,
  outputDir: '../../target/browser-results',
  reporter: [['list'], ['html', { outputFolder: '../../target/browser-report', open: 'never' }]],
  use: { trace: 'retain-on-failure', screenshot: 'only-on-failure' },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'], launchOptions: { args: ['--no-sandbox','--disable-dev-shm-usage'] } } },
    // Temporary workaround for Playwright 1.63 Firefox COOP navigation stalls (#42731).
    // Local portal navigation reproduced stalls that this preference avoided; attribution of
    // the first-navigation CI stall to the same cause remains inferred. Only this test browser
    // disables COOP isolation. Server headers, Rust assertions and Chromium/WebKit stay intact.
    // Remove after a browser/tooling fix is verified with the default preference; #42788 adds
    // regression tests and does not itself establish a product fix or a safe removal version.
    { name: 'firefox', use: { ...devices['Desktop Firefox'], launchOptions: { firefoxUserPrefs: { 'browser.tabs.remote.useCrossOriginOpenerPolicy': false } } } },
    { name: 'webkit', use: { ...devices['Desktop Safari'] } }
  ]
});
