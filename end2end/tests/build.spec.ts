import AxeBuilder from "@axe-core/playwright";
import { expect, Page, test } from "@playwright/test";

const WCAG = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"];

// The full authenticated builder flow needs an admin and a seeded floor. Provide
// E2E_ADMIN_EMAIL / E2E_ADMIN_PASSWORD (the bootstrap admin) to run it; otherwise
// only the unauthenticated gate + a11y run. Watch any of these with `--headed`.
const ADMIN_EMAIL = process.env.E2E_ADMIN_EMAIL;
const ADMIN_PASSWORD = process.env.E2E_ADMIN_PASSWORD;

async function signIn(page: Page): Promise<void> {
  await page.goto("/login");
  await page.locator('input[type="email"], input[name="email"]').first().fill(ADMIN_EMAIL!);
  await page.locator('input[type="password"]').first().fill(ADMIN_PASSWORD!);
  await page.getByRole("button", { name: /sign in|log in/i }).first().click();
}

test.describe("/build — floor builder access", () => {
  test("redirects an unauthenticated visitor to sign in", async ({ page }) => {
    await page.goto("/build");
    // The picker's server fn rejects an unauthenticated load → a Sign in action.
    await expect(page.getByText(/sign in/i).first()).toBeVisible();
  });

  test("an unauthenticated deep link to a floor also gates", async ({ page }) => {
    await page.goto("/build/00000000-0000-0000-0000-000000000000");
    await expect(page.getByText(/sign in/i).first()).toBeVisible();
  });

  test("/build has no WCAG 2.1 AA violations", async ({ page }) => {
    await page.goto("/build");
    await expect(page.getByRole("heading", { name: /floor builder/i })).toBeVisible();
    const results = await new AxeBuilder({ page }).withTags(WCAG).analyze();
    expect(results.violations).toEqual([]);
  });
});

test.describe("/build — authenticated builder flow", () => {
  test.skip(!ADMIN_EMAIL || !ADMIN_PASSWORD, "set E2E_ADMIN_EMAIL/PASSWORD + seed a floor");

  test("place a desk pod, bind a seat, save, and reload shows it", async ({ page }) => {
    await signIn(page);
    await page.goto("/build");
    // Open the first buildable floor.
    await page.getByRole("link").first().click();
    await expect(page.locator('svg[data-slot="floor-builder"], .cn-floor-builder-svg')).toBeVisible();

    // Place a desk pod: select the tool, then click the canvas.
    await page.getByRole("button", { name: /desk pod/i }).click();
    const canvas = page.locator(".cn-floor-builder-svg");
    const box = await canvas.boundingBox();
    if (!box) throw new Error("no canvas");
    await canvas.click({ position: { x: box.width / 2, y: box.height / 2 } });

    // A bookable seat is now selected → the resource panel shows a name field.
    await expect(page.getByRole("button", { name: "Select" })).toBeVisible();
    await page.getByRole("button", { name: "Select" }).click();
    await canvas.click({ position: { x: box.width / 2, y: box.height / 2 } });

    // Save the draft, then reload and confirm the scene persisted.
    await page.getByRole("button", { name: /save draft/i }).click();
    await expect(page.getByText(/saved/i)).toBeVisible();
    await page.reload();
    await expect(page.locator('[data-kind="desk"]').first()).toBeVisible();
  });

  test("the builder canvas + panel have no WCAG 2.1 AA violations", async ({ page }) => {
    await signIn(page);
    await page.goto("/build");
    await page.getByRole("link").first().click();
    await expect(page.locator(".cn-floor-builder-svg")).toBeVisible();
    const results = await new AxeBuilder({ page }).withTags(WCAG).analyze();
    expect(results.violations).toEqual([]);
  });
});
