import { test, expect } from '@playwright/test';

const pcBaseUrl = process.env.DRIVE_E2E_PC_BASE_URL?.replace(/\/$/, '');
const pcSessionJson = process.env.DRIVE_E2E_PC_SESSION_JSON?.trim();
const shareClaimToken = process.env.DRIVE_E2E_PC_SHARE_TOKEN?.trim() || 'e2e-share-token';
const SESSION_STORAGE_KEY = 'sdkwork-drive-pc-session';

test.describe('Drive PC browser smoke', () => {
  test('share claim route renders when staging PC base URL is configured', async ({ page }) => {
    test.skip(!pcBaseUrl, 'Set DRIVE_E2E_PC_BASE_URL for live PC browser smoke');

    if (pcSessionJson) {
      await page.addInitScript(
        ({ storageKey, session }) => {
          localStorage.setItem(storageKey, session);
        },
        { storageKey: SESSION_STORAGE_KEY, session: pcSessionJson },
      );
    }

    const claimPath = `/share/${encodeURIComponent(shareClaimToken)}`;
    const response = await page.goto(`${pcBaseUrl}${claimPath}`);
    expect(response?.ok()).toBeTruthy();

    if (pcSessionJson) {
      await expect(
        page.getByText(/Someone shared a file with you|有人与您分享了文件/),
      ).toBeVisible();
      await expect(page.getByRole('button', { name: /Accept|接受/ })).toBeVisible();
      return;
    }

    const url = new URL(page.url());
    expect(
      url.pathname.includes('/auth/login') || url.pathname.startsWith('/share/'),
    ).toBeTruthy();
    if (url.pathname.includes('/auth/login')) {
      expect(url.searchParams.get('redirect')).toBe(claimPath);
    }
  });
});
