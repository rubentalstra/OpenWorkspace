import { expect, test } from "@playwright/test";

/**
 * Every showcase route. Each must render server-side (HTTP < 400, no SSR panic)
 * and hydrate on the client without throwing.
 */
const ROUTES = [
  "/ui",
  "/ui/buttons",
  "/ui/inputs",
  "/ui/forms",
  "/ui/overlays",
  "/ui/navigation",
  "/ui/data",
  "/ui/dates",
  "/ui/feedback",
  "/ui/layout",
  "/ui/theme",
  "/ui/hooks",
];

/** Console-error text we treat as benign (asset/network noise, not app bugs). */
const IGNORED_CONSOLE = [/favicon/i, /Failed to load resource/i, /net::ERR/i];

test.describe("/ui renders and hydrates without errors", () => {
  for (const route of ROUTES) {
    test(`${route} loads clean`, async ({ page }) => {
      const errors: string[] = [];
      page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
      page.on("console", (m) => {
        if (m.type() !== "error") return;
        const text = m.text();
        if (IGNORED_CONSOLE.some((re) => re.test(text))) return;
        errors.push(`console.error: ${text}`);
      });

      const resp = await page.goto(route, { waitUntil: "networkidle" });
      expect(resp, `no response for ${route}`).not.toBeNull();
      expect(resp!.status(), `HTTP status for ${route}`).toBeLessThan(400);

      // Server rendered the page frame.
      await expect(page.locator("h1").first()).toBeVisible();

      // Hydration ran: the sidenav trigger is a button wired to toggle its
      // pressed state via wasm. A static (un-hydrated) page would never update
      // aria-pressed on click.
      const trigger = page.locator('[data-name="SidenavTrigger"]').first();
      await expect(trigger).toBeVisible();
      const before = await trigger.getAttribute("aria-pressed");
      await trigger.click();
      await expect
        .poll(() => trigger.getAttribute("aria-pressed"), {
          message: `sidenav trigger never toggled on ${route} — hydration did not run`,
        })
        .not.toBe(before);

      expect(errors, `runtime errors on ${route}`).toEqual([]);
    });
  }
});
