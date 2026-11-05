// Playwright E2E tests for EKPI governance web
// Run with: npx playwright test

import { test, expect } from '@playwright/test';

test.describe('EKPI Governance Workbench', () => {
  test('health endpoint returns 200', async ({ request }) => {
    const response = await request.get('http://localhost:8080/healthz');
    expect(response.status()).toBe(200);
    const body = await response.json();
    expect(body.status).toBe('ok');
  });

  test('metrics endpoint returns prometheus format', async ({ request }) => {
    const response = await request.get('http://localhost:8080/metrics');
    expect(response.status()).toBe(200);
    const text = await response.text();
    expect(text).toContain('ekpi_http_requests_total');
  });

  test('login page renders', async ({ page }) => {
    await page.goto('http://localhost:8080/login');
    await expect(page.locator('h2')).toContainText('本機登入');
    await expect(page.locator('input[name="username"]')).toBeVisible();
    await expect(page.locator('input[name="password"]')).toBeVisible();
  });

  test('unauthenticated access redirects to login', async ({ page }) => {
    await page.goto('http://localhost:8080/projects');
    await expect(page).toHaveURL(/login/);
  });

  test('stylesheet loads', async ({ request }) => {
    const response = await request.get('http://localhost:8080/assets/governance.css');
    expect(response.status()).toBe(200);
    const contentType = response.headers()['content-type'];
    expect(contentType).toContain('text/css');
  });
});
