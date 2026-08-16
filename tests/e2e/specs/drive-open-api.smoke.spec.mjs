import { test, expect } from '@playwright/test';

const baseUrl = process.env.DRIVE_E2E_OPEN_API_BASE_URL?.replace(/\/$/, '');

test.describe('Drive open-api production smoke', () => {
  test('healthz responds when staging base URL is configured', async ({ request }) => {
    test.skip(!baseUrl, 'Set DRIVE_E2E_OPEN_API_BASE_URL for live open-api smoke');

    const response = await request.get(`${baseUrl}/healthz`);
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.status).toBe('ok');
    expect(body.service).toBe('sdkwork-routes-drive-open-api');
  });

  test('metrics exposes drive histogram when staging base URL is configured', async ({ request }) => {
    test.skip(!baseUrl, 'Set DRIVE_E2E_OPEN_API_BASE_URL for live open-api smoke');

    const response = await request.get(`${baseUrl}/metrics`);
    expect(response.ok()).toBeTruthy();
    const body = await response.text();
    expect(body).toContain('drive_http_request_duration_seconds_bucket');
    expect(body).toContain('drive_health_status');
  });

  test('share link resolve enforces extraction code on staging', async ({ request }) => {
    const token = process.env.DRIVE_E2E_SHARE_TOKEN;
    const accessCode = process.env.DRIVE_E2E_SHARE_ACCESS_CODE;
    test.skip(
      !baseUrl || !token || !accessCode,
      'Set DRIVE_E2E_OPEN_API_BASE_URL, DRIVE_E2E_SHARE_TOKEN, DRIVE_E2E_SHARE_ACCESS_CODE',
    );

    const denied = await request.get(`${baseUrl}/open/v3/api/drive/share_links/${token}`);
    expect(denied.status()).toBe(403);
    const deniedBody = await denied.json();
    const serverTraceId = denied.headers()['x-sdkwork-trace-id'];
    expect(deniedBody.code).toBe(40301);
    expect(typeof deniedBody.traceId).toBe('string');
    expect(deniedBody.traceId.length).toBeGreaterThan(0);
    expect(serverTraceId).toBe(deniedBody.traceId);

    const allowed = await request.get(
      `${baseUrl}/open/v3/api/drive/share_links/${token}?accessCode=${encodeURIComponent(accessCode)}`,
    );
    expect(allowed.ok()).toBeTruthy();
    const allowedBody = await allowed.json();
    expect(allowedBody.accessCodeRequired).toBe(true);
  });
});
