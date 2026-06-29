import AxeBuilder from "@axe-core/playwright";
import { expect, Page, test } from "@playwright/test";

const WCAG = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"];

const floorSvg = (page: Page) => page.locator('svg[data-slot="floor-plan"]');

/**
 * Gate on hydration: the renderer's `viewBox` only changes once the wheel handler
 * is wired (SSR ships a static `viewBox`). Retry a zoom until it actually moves.
 */
async function waitForFloorHydration(page: Page): Promise<void> {
  const svg = floorSvg(page);
  await expect(svg).toBeVisible();
  await svg.hover();
  await expect(async () => {
    const before = await svg.getAttribute("viewBox");
    await page.mouse.wheel(0, -120);
    const after = await svg.getAttribute("viewBox");
    expect(after).not.toBe(before);
  }).toPass({ timeout: 15000 });
}

test.describe("/ui/floor — floor plan renderer", () => {
  test("server-renders and hydrates without console errors", async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (m) => {
      if (m.type() === "error") errors.push(m.text());
    });
    page.on("pageerror", (e) => errors.push(e.message));

    await page.goto("/ui/floor");
    // SSR emits the static SVG (group + bookable desks) before hydration.
    await expect(floorSvg(page)).toBeVisible();
    await expect(page.locator('[data-kind="desk"][role="button"]').first()).toBeVisible();

    await waitForFloorHydration(page);
    expect(errors).toEqual([]);
  });

  test("drag pans the viewBox", async ({ page }) => {
    await page.goto("/ui/floor");
    const svg = floorSvg(page);
    await waitForFloorHydration(page);

    const box = await svg.boundingBox();
    expect(box).not.toBeNull();
    if (!box) return;
    const cx = box.x + box.width / 2;
    const cy = box.y + box.height / 2;

    const before = await svg.getAttribute("viewBox");
    await page.mouse.move(cx, cy);
    await page.mouse.down();
    await page.mouse.move(cx + 60, cy + 40, { steps: 4 });
    await page.mouse.up();
    await expect.poll(() => svg.getAttribute("viewBox")).not.toBe(before);
  });

  test("bookable desks are keyboard focusable", async ({ page }) => {
    await page.goto("/ui/floor");
    await waitForFloorHydration(page);
    const desk = page.locator('[data-kind="desk"][role="button"]').first();
    await desk.focus();
    await expect(desk).toBeFocused();
  });

  test("selecting a desk updates the selection readout", async ({ page }) => {
    await page.goto("/ui/floor");
    await waitForFloorHydration(page);
    await page.locator('[data-kind="desk"][role="button"]').first().click();
    await expect(page.getByText(/^Selected:/)).toBeVisible();
  });

  test("has no WCAG 2.1 AA violations", async ({ page }) => {
    await page.goto("/ui/floor");
    await expect(floorSvg(page)).toBeVisible();
    const results = await new AxeBuilder({ page }).withTags(WCAG).analyze();
    expect(results.violations).toEqual([]);
  });
});
