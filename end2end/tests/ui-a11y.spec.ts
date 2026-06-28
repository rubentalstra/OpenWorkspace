import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const WCAG = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"];

/** Theme-toggle button in the `/ui` showcase header. */
const themeToggle = (page: import("@playwright/test").Page) =>
  page.getByRole("button", { name: "Toggle dark mode" });

test.describe("UI showcase accessibility", () => {
  test("has no WCAG 2.1 AA violations in light mode", async ({ page }) => {
    await page.goto("/ui");
    const results = await new AxeBuilder({ page }).withTags(WCAG).analyze();
    expect(results.violations).toEqual([]);
  });

  test("has no WCAG 2.1 AA violations in dark mode", async ({ page }) => {
    await page.goto("/ui");
    await themeToggle(page).click();
    await expect(page.locator("html")).toHaveClass(/dark/);
    const results = await new AxeBuilder({ page }).withTags(WCAG).analyze();
    expect(results.violations).toEqual([]);
  });

  test("theme toggle exposes its state via aria-pressed", async ({ page }) => {
    await page.goto("/ui");
    const toggle = themeToggle(page);
    await expect(toggle).toHaveAttribute("aria-pressed", "false");
    await toggle.click();
    await expect(toggle).toHaveAttribute("aria-pressed", "true");
  });
});
