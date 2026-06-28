import { test } from "@playwright/test";

test("sidebar expanded then collapsed", async ({ page }) => {
  await page.goto("/ui/buttons", { waitUntil: "networkidle" });
  await page.waitForTimeout(1800); // let wasm hydrate
  await page.screenshot({ path: "/tmp/shot-expanded.png" });

  const trigger = page.locator('[data-name="SidenavTrigger"]').first();
  await trigger.click();
  await page.waitForTimeout(600);
  await page.screenshot({ path: "/tmp/shot-collapsed.png" });

  // hover the first collapsed nav icon to surface a tooltip/title
  await page.locator('[data-name="SidenavMenuButton"]').first().hover();
  await page.waitForTimeout(400);
  await page.screenshot({ path: "/tmp/shot-collapsed-hover.png" });
});
