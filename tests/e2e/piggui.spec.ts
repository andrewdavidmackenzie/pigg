import { test, expect } from '@playwright/test';
import { spawn, ChildProcess, execSync } from 'child_process';

// --- Smoke Tests ---

test.describe('Smoke tests', () => {
  test('page loads without console errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') errors.push(msg.text());
    });

    await page.goto('/');
    await page.waitForTimeout(3000);

    const realErrors = errors.filter(
      e => !e.includes('integrity') && !e.includes('favicon')
    );
    expect(realErrors).toEqual([]);
  });

  test('canvas element exists with non-zero dimensions', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(3000);

    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();

    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.width).toBeGreaterThan(0);
    expect(box!.height).toBeGreaterThan(0);
  });

  test('canvas renders content', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(5000);

    // wgpu uses WebGL/WebGPU context so we can't sample pixels directly.
    // Take a screenshot and verify it has some content — any rendered UI
    // produces a PNG larger than a trivial empty image.
    const canvas = page.locator('canvas');
    const screenshot = await canvas.screenshot();
    expect(screenshot.length).toBeGreaterThan(500);
  });
});

// --- UI Interaction Tests ---

test.describe('UI interaction tests', () => {
  test('disconnected view shows interactive canvas', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(3000);

    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();

    // Verify canvas has tabindex (iced sets this for keyboard focus)
    const tabindex = await canvas.getAttribute('tabindex');
    expect(tabindex).toBe('0');
  });

  // Note: iced renders everything to a canvas, so we cannot query widget state
  // or verify text content directly. We verify that keyboard input is processed
  // without errors, which confirms the iced event loop is running.
  test('keyboard input is accepted by canvas', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(3000);

    const canvas = page.locator('canvas');
    await canvas.click();

    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') errors.push(msg.text());
    });

    await canvas.press('a');
    await canvas.press('b');
    await canvas.press('c');
    await page.waitForTimeout(500);

    const realErrors = errors.filter(
      e => !e.includes('integrity') && !e.includes('favicon')
    );
    expect(realErrors).toEqual([]);
  });
});

// --- Connectivity Tests ---

test.describe('Connectivity tests', () => {
  let pigglet: ChildProcess;
  let endpointId: string;

  // Increase beforeAll timeout for cargo build on CI
  test.beforeAll(async () => {
    test.setTimeout(180_000);

    try { execSync('pkill -f "target/debug/pigglet"', { stdio: 'ignore' }); } catch {}
    await new Promise(r => setTimeout(r, 1000));

    execSync('cargo build -p pigglet', {
      cwd: '../..',
      stdio: 'inherit',
      timeout: 120_000,
    });

    pigglet = spawn('cargo', ['run', '-p', 'pigglet', '--bin', 'pigglet'], {
      cwd: '../..',
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    endpointId = await new Promise<string>((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('Timeout waiting for pigglet endpoint_id')), 30_000);
      let output = '';

      pigglet.stdout!.on('data', (data: Buffer) => {
        output += data.toString();
        const match = output.match(/endpoint_id: ([a-f0-9]{64})/);
        if (match) {
          clearTimeout(timeout);
          resolve(match[1]);
        }
      });

      pigglet.stderr!.on('data', (data: Buffer) => {
        output += data.toString();
      });

      pigglet.on('error', (err) => {
        clearTimeout(timeout);
        reject(err);
      });
    });
  });

  test.afterAll(async () => {
    if (pigglet) {
      pigglet.kill('SIGTERM');
      await new Promise(r => setTimeout(r, 1000));
    }
  });

  // This test requires iroh relay connectivity which may be slow or
  // unavailable on CI runners. Skip if CI environment detected.
  const isCI = !!process.env.CI;
  (isCI ? test.skip : test)('auto-connect via URL parameter', async ({ page }) => {
    test.setTimeout(90_000);

    // Listen for pigglet to confirm the connection
    const connectedPromise = new Promise<boolean>((resolve) => {
      const timeout = setTimeout(() => resolve(false), 60_000);
      pigglet.stdout!.on('data', (data: Buffer) => {
        if (data.toString().includes('Connection via Iroh')) {
          clearTimeout(timeout);
          resolve(true);
        }
      });
    });

    await page.goto(`/?endpoint_id=${endpointId}`);

    // Wait for pigglet to confirm connection
    const connected = await connectedPromise;
    expect(connected).toBe(true);

    // Verify canvas is still rendering after connection
    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();

    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.width).toBeGreaterThan(0);
    expect(box!.height).toBeGreaterThan(0);
  });
});
