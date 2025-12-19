import { test, expect } from '@playwright/test';

test('placeholder homepage loads', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await expect(page.locator('text=BFF MVP')).toBeVisible();
});
