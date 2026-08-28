// SPDX-License-Identifier: Apache-2.0
import { expect, test as base } from '@playwright/test';
import { readdirSync } from 'node:fs';
import { dirname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { auditDocument } from '../scripts/audit-export-dom.mjs';

const siteDir = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const outDir = join(siteDir, 'out');
const viewports = [320, 768, 1440];
const expectedDiagramNames = [
  'Capture packet attribution architecture',
  'Deep Capture session architecture',
];

const test = base.extend({
  browserErrors: [async ({ page }, use) => {
    const errors = [];
    page.on('pageerror', (error) => {
      errors.push(`${new URL(page.url()).pathname}: page: ${error.message}`);
    });
    page.on('console', (message) => {
      if (message.type() !== 'error') return;
      const location = message.location();
      const expectedMissingDocument = location.url === page.url()
        && /^Failed to load resource: the server responded with a status of 404 \(Not Found\)$/.test(message.text());
      if (expectedMissingDocument) return;
      errors.push(
        `${new URL(page.url()).pathname}: console: ${message.text()} (${location.url || 'unknown URL'})`,
      );
    });

    await use(errors);
    await page.waitForTimeout(50);
    expect(errors, 'browser and console errors').toEqual([]);
  }, { auto: true }],
});

function exportedHtmlFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? exportedHtmlFiles(path) : [path];
  });
}

function publicRoutes() {
  return exportedHtmlFiles(outDir)
    .filter((path) => path.endsWith('.html'))
    .map((path) => relative(outDir, path).split(sep).join('/'))
    .filter((path) => path !== '404.html' && path !== '_not-found.html')
    .map((path) => {
      const withoutExtension = path.slice(0, -'.html'.length);
      return withoutExtension === 'index' ? '/' : `/${withoutExtension}`;
    })
    .sort();
}

function contrastRatio(foreground, background) {
  const linear = (value) => {
    const channel = value / 255;
    return channel <= 0.04045
      ? channel / 12.92
      : ((channel + 0.055) / 1.055) ** 2.4;
  };
  const luminance = (color) => (
    0.2126 * linear(color[0])
    + 0.7152 * linear(color[1])
    + 0.0722 * linear(color[2])
  );
  const lighter = Math.max(luminance(foreground), luminance(background));
  const darker = Math.min(luminance(foreground), luminance(background));
  return (lighter + 0.05) / (darker + 0.05);
}

function rgb(value) {
  const channels = value.match(/[\d.]+/g)?.slice(0, 3).map(Number);
  if (!channels || channels.length !== 3) {
    throw new Error(`expected an RGB color, received ${value}`);
  }
  return channels;
}

async function openSearch(page, query) {
  await page.goto('/docs/getting-started');
  await page.locator('[data-search-full]:visible, [data-search]:visible').first().click();
  const dialog = page.getByRole('dialog', { name: 'Search' });
  await dialog.getByPlaceholder('Search').fill(query);
  const results = dialog.locator('button[aria-selected]');
  await page.waitForTimeout(500);
  await expect(results.first(), `${query} result population`).toBeVisible();
  expect(await results.count(), `${query} result count`).toBeGreaterThan(0);
  return { dialog, results };
}

async function activateFirstSearchResult(page, query, destination) {
  const { results } = await openSearch(page, query);
  await results.first().click();
  await expect(page).toHaveURL((url) => url.pathname === destination);
}

test.describe('production accessibility contract', () => {
  test.beforeEach(async ({ context, page }) => {
    await context.addInitScript(() => localStorage.setItem('theme', 'light'));
    await page.emulateMedia({ colorScheme: 'light', reducedMotion: 'reduce' });
  });

  for (const width of viewports) {
    test(`all public routes expose one primary region and keyboard bypass at ${width}px`, async ({ page }) => {
      const routes = publicRoutes();
      expect(routes, 'public route population').toHaveLength(54);
      await page.setViewportSize({ width, height: 900 });
      const skipRequests = [];
      page.on('request', (request) => skipRequests.push(request.url()));

      for (const route of routes) {
        const response = await page.goto(route);
        expect(response?.status(), `${route} response`).toBe(200);

        const observation = await page.evaluate(auditDocument);
        expect(observation.mainCount, `${route} primary landmark count`).toBe(1);
        expect(observation.primaryId, `${route} primary landmark identity`).toBe('main-content');
        expect(observation.skipHref, `${route} skip destination`).toBe('#main-content');
        expect(observation.firstFocusableClass, `${route} first focusable class`).toContain('fc-skip-link');

        await page.locator('.fc-skip-link').focus();
        const skipBox = await page.locator('.fc-skip-link').boundingBox();
        expect(skipBox, `${route} focused skip bounds`).not.toBeNull();
        expect(skipBox.x, `${route} focused skip left`).toBeGreaterThanOrEqual(0);
        expect(skipBox.y, `${route} focused skip top`).toBeGreaterThanOrEqual(0);
        expect(skipBox.x + skipBox.width, `${route} focused skip right`).toBeLessThanOrEqual(width);

        await page.waitForTimeout(50);
        skipRequests.length = 0;
        await page.locator('.fc-skip-link').press('Enter');
        await expect.poll(
          () => page.evaluate(() => ({
            hash: location.hash,
            activeId: document.activeElement?.id,
          })),
          { message: `${route} skip activation` },
        ).toEqual({ hash: '#main-content', activeId: 'main-content' });
        expect(skipRequests, `${route} skip activation requests`).toEqual([]);
      }
    });
  }

  test('generated changelog headings preserve hierarchy and anchors', async ({ page }) => {
    const routes = publicRoutes().filter((route) => route.startsWith('/docs/changelog/'));
    expect(routes.length, 'generated changelog route population').toBeGreaterThan(0);

    for (const route of routes) {
      await page.goto(route);
      const observation = await page.evaluate(auditDocument);
      expect(observation.headingSkips, `${route} heading skips`).toEqual([]);
    }

    await page.goto('/docs/changelog/0-5-0/highlights');
    await expect(page.locator('#installing-on-windows')).toHaveCount(1);
    expect(await page.locator('a[href="#installing-on-windows"]').count()).toBeGreaterThan(0);
  });

  test('light-theme muted and syntax text meet normal-text contrast', async ({ page }) => {
    await page.goto('/docs/getting-started');
    const subjects = await page.evaluate(() => {
      const visible = (element) => {
        const box = element.getBoundingClientRect();
        const style = getComputedStyle(element);
        return box.width > 0 && box.height > 0 && style.visibility !== 'hidden';
      };
      const opaqueBackground = (element) => {
        let current = element;
        while (current) {
          const color = getComputedStyle(current).backgroundColor;
          const transparent = color === 'rgba(0, 0, 0, 0)'
            || color.endsWith(', 0)')
            || /\/\s*(?:0(?:\.\d+)?|\d?\.\d+)\s*\)$/.test(color);
          if (color && !transparent) return color;
          current = current.parentElement;
        }
        return getComputedStyle(document.body).backgroundColor;
      };
      const gather = (selector) => [...document.querySelectorAll(selector)]
        .filter(visible)
        .filter((element) => (element.textContent || '').trim())
        .map((element) => ({
          foreground: getComputedStyle(element).color,
          background: opaqueBackground(element),
          text: (element.textContent || '').trim().slice(0, 60),
        }));
      return {
        muted: gather('.text-fd-muted-foreground'),
        syntax: gather('[style*="--shiki-light:#CC3346" i]'),
      };
    });

    expect(subjects.muted.length, 'visible muted text population').toBeGreaterThan(0);
    expect(subjects.syntax.length, 'corrected syntax text population').toBeGreaterThan(0);
    const correctedMuted = subjects.muted.filter(
      (subject) => rgb(subject.foreground).join(',') === '110,110,110',
    );
    expect(correctedMuted.length, 'corrected muted token population').toBeGreaterThan(0);
    for (const subject of subjects.syntax) {
      expect(rgb(subject.foreground), subject.text).toEqual([204, 51, 70]);
    }
    for (const subject of [...subjects.muted, ...subjects.syntax]) {
      expect(
        contrastRatio(rgb(subject.foreground), rgb(subject.background)),
        `${subject.foreground} on ${subject.background}: ${subject.text}`,
      ).toBeGreaterThanOrEqual(4.5);
    }
  });

  for (const width of viewports) {
    test(`architecture diagrams expose distinct names at ${width}px`, async ({ page }) => {
      await page.setViewportSize({ width, height: 900 });
      await page.goto('/docs/architecture');
      await expect(page.locator('article .mermaid svg')).toHaveCount(2);
      const diagrams = await page.locator('article .mermaid svg').evaluateAll((elements) => (
        elements.map((element) => {
          const labelledBy = element.getAttribute('aria-labelledby');
          const labelled = labelledBy
            ? labelledBy.split(/\s+/).map((id) => document.getElementById(id)?.textContent || '').join(' ')
            : '';
          return {
            role: element.getAttribute('role'),
            name: (element.getAttribute('aria-label') || labelled || element.querySelector('title')?.textContent || '').trim(),
          };
        })
      ));
      expect(diagrams.map((diagram) => diagram.name)).toEqual(expectedDiagramNames);
      expect(new Set(diagrams.map((diagram) => diagram.name)).size).toBe(2);
      for (const diagram of diagrams) {
        expect(diagram.role).toContain('graphics-document');
      }
    });
  }

  test('retired command searches lead to current guidance and retain history', async ({ page }) => {
    const cases = [
      { query: 'fragcap run', replacement: 'fragcap capture' },
      { query: '  FRAGCAP RUN  ', replacement: 'fragcap capture' },
      { query: 'fragcap tap', replacement: 'fragcap capture --process' },
      { query: '  FrAgCaP TaP  ', replacement: 'fragcap capture --process' },
    ];

    for (const { query, replacement } of cases) {
      const { results } = await openSearch(page, query);
      const labels = await results.allInnerTexts();
      const historicalIndex = labels.findIndex((label) => (
        /Changelog\s+0\.5\.0/i.test(label.replace(/\s+/g, ' '))
      ));
      expect(historicalIndex, `${query} preserved historical group`).toBeGreaterThan(0);

      await results.first().click();
      await expect(page).toHaveURL((url) => url.pathname === '/docs/reference/cli');
      await expect(page.getByRole('main')).toContainText(replacement);
    }
  });

  test('current guidance searches retain their leading destinations', async ({ page }) => {
    const cases = [
      ['packet attribution', '/docs/reference/deep-capture-compatibility'],
      ['capture scope', '/docs/architecture'],
      ['Deep Capture', '/docs/architecture'],
      ['proxy-owned TLS key', '/docs/reference/output-formats'],
    ];

    for (const [query, destination] of cases) {
      await activateFirstSearchResult(page, query, destination);
    }
  });

  for (const width of [320, 1440]) {
    test(`not-found responses provide recovery at ${width}px`, async ({ page }) => {
      await page.setViewportSize({ width, height: 900 });
      const missingPaths = [
        '/definitely-missing-s096',
        '/docs/reference/no/such/route-s096',
      ];

      for (const missingPath of missingPaths) {
        const response = await page.goto(missingPath);
        expect(response?.status(), `${missingPath} response`).toBe(404);
        await expect(page.locator('main#main-content')).toHaveCount(1);
        await expect(page.getByRole('heading', { level: 1, name: 'Page not found' })).toHaveCount(1);
        await expect(page.locator('main#main-content')).toContainText('fragcap');

        const recovery = page.getByRole('navigation', { name: 'Page recovery' });
        const links = recovery.getByRole('link');
        await expect(links).toHaveCount(2);
        await expect(links.nth(0)).toHaveAttribute('href', '/');
        await expect(links.nth(1)).toHaveAttribute('href', '/docs/getting-started');
        for (const link of await links.all()) {
          await expect(link).toBeVisible();
          await link.focus();
          await expect(link).toBeFocused();
          const box = await link.boundingBox();
          expect(box, `${missingPath} recovery link bounds`).not.toBeNull();
          expect(box.x, `${missingPath} recovery link left`).toBeGreaterThanOrEqual(0);
          expect(box.x + box.width, `${missingPath} recovery link right`).toBeLessThanOrEqual(width);
        }

        expect(
          await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth + 2),
          `${missingPath} root overflow`,
        ).toBe(true);

        await page.reload();
        await page.keyboard.press('Tab');
        await expect(page.locator('.fc-skip-link')).toBeFocused();
        await page.locator('.fc-skip-link').press('Enter');
        await expect.poll(
          () => page.evaluate(() => ({
            hash: location.hash,
            activeId: document.activeElement?.id,
          })),
          { message: `${missingPath} skip activation` },
        ).toEqual({ hash: '#main-content', activeId: 'main-content' });

        await links.nth(0).click();
        await expect(page).toHaveURL((url) => url.pathname === '/');
        await page.goto(missingPath);
        await page.getByRole('navigation', { name: 'Page recovery' }).getByRole('link').nth(1).click();
        await expect(page).toHaveURL((url) => url.pathname === '/docs/getting-started');
      }
    });
  }
});
