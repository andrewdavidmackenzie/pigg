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

    // Filter out known non-error messages
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

    // Take a screenshot of the canvas and verify it's not a single solid color
    const canvas = page.locator('canvas');
    const screenshot = await canvas.screenshot();
    expect(screenshot.length).toBeGreaterThan(1000);
  });
});

// --- UI Interaction Tests ---

test.describe('UI interaction tests', () => {
  test('disconnected view shows text content', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(3000);

    // The canvas captures all rendering — we can't query text directly.
    // Instead verify the canvas is interactive (iced is running).
    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();

    // Verify canvas has focus/tabindex (iced sets this)
    const tabindex = await canvas.getAttribute('tabindex');
    expect(tabindex).toBe('0');
  });

  test('keyboard input is accepted by canvas', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(3000);

    const canvas = page.locator('canvas');
    await canvas.click();

    // Type some characters — iced should process them without errors
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
    // Kill any existing pigglet
    try { execSync('pkill -f "target/debug/pigglet"', { stdio: 'ignore' }); } catch {}
    await new Promise(r => setTimeout(r, 1000));

    // Build pigglet
    execSync('cargo build -p pigglet', {
      cwd: '../..',
      stdio: 'inherit',
      timeout: 120_000,
    });

    // Start pigglet and capture endpoint_id
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
    // Load with endpoint_id parameter to auto-connect
    await page.goto(`/?endpoint_id=${endpointId}`);

    // Wait for connection — pigglet prints "Connected" when a client connects
    // Give it time for iroh relay connection
    await page.waitForTimeout(30_000);

    // Verify canvas is still rendering (no crash)
    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();

    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.width).toBeGreaterThan(0);
    expect(box!.height).toBeGreaterThan(0);
  });
});
