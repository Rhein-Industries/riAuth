import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { fixtureStartupMs, startFixture } from './fixture.js';
let stopFixture, fixture;
test.beforeAll(async () => {
  test.setTimeout(fixtureStartupMs + 5000);
  ({ fixture, stop: stopFixture } = await startFixture());
});
test.afterAll(async () => { await stopFixture?.(); });
async function accessibility(page) {
  const report = await new AxeBuilder({page}).withTags(['wcag2a','wcag2aa','wcag21aa']).analyze();
  expect(report.violations.map(({id,nodes}) => ({id,nodes:nodes.map(({target,failureSummary})=>({target,failureSummary}))}))).toEqual([]);
}
test('terminal approval, cancellation, keyboard access, reflow and private-context logout', async ({page, context}) => {
  await page.goto(`${fixture.issuer}/apps`);
  const start = page.getByRole('button', {name:/sign in with your terminal/i});
  await expect(start).toBeVisible();
  await expect(page.getByRole('heading', {name:'Sign in', exact:true})).toBeVisible();
  await expect(page).toHaveTitle('Sign in · riAuth');
  await expect(page.getByRole('complementary', {name:'Workspace navigation'})).toBeHidden();
  await expect(page.locator('#workspace-header')).toBeHidden();
  await expect(page.getByRole('button', {name:'All applications', exact:false})).toBeHidden();
  await expect(page.getByRole('searchbox', {name:'Search applications'})).toBeHidden();
  for (const width of [1280, 640, 320]) {
    await page.setViewportSize({width,height:900});
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await expect(start).toBeVisible();
  }
  await accessibility(page);
  await page.setViewportSize({width:1280,height:900});
  await start.focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('#user-code')).not.toBeEmpty();
  await page.getByRole('button', {name:'Cancel sign-in'}).click();
  await expect(start).toBeVisible();
  await start.click();
  await expect(page.locator('#user-code')).not.toBeEmpty();
  const code = await page.locator('#user-code').innerText();
  const response = await page.request.post(`${fixture.issuer}/api/portal/requests/${encodeURIComponent(code)}`, {headers:{authorization:`Bearer ${fixture.token}`}, data:{approve:true}});
  expect(response.ok()).toBe(true);
  await expect(page.getByRole('link',{name:'Open Fixture application (opens in a new tab)',exact:true})).toBeVisible({timeout:15000});
  await expect(page.getByRole('complementary', {name:'Workspace navigation'})).toBeVisible();
  await expect(page.locator('#workspace-header')).toBeVisible();
  await expect(page).toHaveTitle('Your applications · riAuth');
  await accessibility(page);
  for (const width of [1280, 640, 320]) {
    await page.setViewportSize({width,height:900});
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  }
  await page.setViewportSize({width:1280,height:900});
  // 200% text scaling independently of viewport reflow.
  await page.evaluate(() => { for (const node of document.querySelectorAll('h1,h2,h3,p,a,button,input,select')) node.style.fontSize = `${parseFloat(getComputedStyle(node).fontSize)*2}px`; });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.reload();
  await expect(page.getByRole('searchbox',{name:'Search applications'})).toBeVisible();
  await page.keyboard.press('/');
  await expect(page.getByRole('searchbox',{name:'Search applications'})).toBeFocused();
  await context.setOffline(true);
  await page.getByRole('button',{name:'Refresh applications'}).click();
  await expect(page.locator('#connection-label')).toContainText(/offline|reconnect|interrupted/i);
  await expect(page.locator('#workspace-header')).toBeHidden();
  await expect(page.getByRole('button',{name:'Try again'})).toBeVisible();
  await context.setOffline(false);
  await page.getByRole('button',{name:'Try again'}).click();
  await expect(page.getByRole('link',{name:'Open Fixture application (opens in a new tab)',exact:true})).toBeVisible();
  await page.getByRole('button',{name:'Sign out'}).focus();
  await expect(page.getByRole('button',{name:'Sign out'})).toBeEnabled();
  await page.keyboard.press('Enter');
  await expect(start).toBeVisible();
  await expect(page.getByRole('heading', {name:'Sign in', exact:true})).toBeFocused();
  await expect(page.getByRole('complementary', {name:'Workspace navigation'})).toBeHidden();
  await expect(page.locator('#workspace-header')).toBeHidden();
  await expect(page).toHaveTitle('Sign in · riAuth');
  await expect(page.getByRole('link',{name:'Open Fixture application (opens in a new tab)',exact:true})).toHaveCount(0);
});
