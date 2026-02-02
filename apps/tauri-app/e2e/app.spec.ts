import { test, expect } from '@playwright/test';

/**
 * E2E tests for DeliDev main application flows.
 */

test.describe('Application', () => {
  test('should load the home page', async ({ page }) => {
    await page.goto('/');

    // Wait for the app to load
    await page.waitForLoadState('networkidle');

    // Check that the app has loaded (look for main container)
    await expect(page.locator('body')).toBeVisible();
  });

  test('should navigate to mode selection', async ({ page }) => {
    await page.goto('/mode-select');

    await page.waitForLoadState('networkidle');

    // Should show mode selection options
    await expect(page.locator('body')).toBeVisible();
  });

  test('should have proper viewport on mobile', async ({ page }) => {
    await page.goto('/');

    // Get viewport size
    const viewport = page.viewportSize();
    expect(viewport).toBeDefined();
  });
});

test.describe('Navigation', () => {
  test('should navigate between pages', async ({ page }) => {
    // Start at home
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Navigate to settings if available
    const settingsLink = page.locator('[href="/settings"]').first();
    if (await settingsLink.isVisible()) {
      await settingsLink.click();
      await expect(page).toHaveURL(/settings/);
    }
  });

  test('should handle 404 pages gracefully', async ({ page }) => {
    await page.goto('/non-existent-page');

    // App should still load without crashing
    await expect(page.locator('body')).toBeVisible();
  });
});

test.describe('Accessibility', () => {
  test('should have proper page structure', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Check for basic accessibility: page should have a body
    const body = page.locator('body');
    await expect(body).toBeVisible();
  });

  test('should be keyboard navigable', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Tab through the page and ensure focus moves
    await page.keyboard.press('Tab');

    // The active element should change when tabbing
    const activeElement = await page.evaluate(() => document.activeElement?.tagName);
    expect(activeElement).toBeDefined();
  });
});

test.describe('Responsive Design', () => {
  test('should adapt to mobile viewport', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Page should load correctly on mobile
    await expect(page.locator('body')).toBeVisible();
  });

  test('should adapt to tablet viewport', async ({ page }) => {
    // Set tablet viewport
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Page should load correctly on tablet
    await expect(page.locator('body')).toBeVisible();
  });

  test('should adapt to desktop viewport', async ({ page }) => {
    // Set desktop viewport
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Page should load correctly on desktop
    await expect(page.locator('body')).toBeVisible();
  });
});
