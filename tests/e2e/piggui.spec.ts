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

  test('canvas renders non-uniform content', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(3000);

    // Sample pixels from the canvas to verify non-uniform rendering
    const hasVariedContent = await page.evaluate(() => {
      const canvas = document.querySelector('canvas');
      if (!canvas) return false;
      const ctx = canvas.getContext('2d', { willReadFrequently: true });
      if (!ctx) return false;
      const w = canvas.width;
      const h = canvas.height;
      // Sample a few rows of pixels
      const topRow = ctx.getImageData(0, 10, w, 1).data;
      const midRow = ctx.getImageData(0, Math.floor(h / 2), w, 1).data;
      // Check if any pixel differs from the first pixel in each row
      const firstR = topRow[0], firstG = topRow[1], firstB = topRow[2];
      for (let i = 4; i < topRow.length; i += 4) {
        if (topRow[i] !== firstR || topRow[i+1] !== firstG || topRow[i+2] !== firstB) return true;
      }
      for (let i = 0; i < midRow.length; i += 4) {
        if (midRow[i] !== firstR || midRow[i+1] !== firstG || midRow[i+2] !== firstB) return true;
      }
      return false;
    });
    expect(hasVariedContent).toBe(true);
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

  test.beforeAll(async () => {
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

  test('auto-connect via URL parameter', async ({ page }) => {
    // Listen for pigglet to confirm the connection
    const connectedPromise = new Promise<boolean>((resolve) => {
      const timeout = setTimeout(() => resolve(false), 45_000);
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
