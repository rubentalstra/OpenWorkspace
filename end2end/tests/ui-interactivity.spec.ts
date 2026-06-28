import { expect, Page, test } from "@playwright/test";

/**
 * The debug wasm bundle finishes hydrating slightly after `networkidle`, so a
 * click fired too early is silently dropped. Gate every test on hydration being
 * live: retry clicking the shell's sidenav trigger until its `aria-pressed`
 * actually toggles (only wired-up wasm does that), then restore it.
 */
async function waitForHydration(page: Page): Promise<void> {
  const trigger = page.locator('[data-name="SidenavTrigger"]').first();
  await expect(async () => {
    const before = await trigger.getAttribute("aria-pressed");
    await trigger.click();
    const after = await trigger.getAttribute("aria-pressed");
    expect(after).not.toBe(before);
  }).toPass({ timeout: 15000 });
  await trigger.click(); // restore prior state
}

test.describe("/ui interactivity (hydration + wired handlers)", () => {
  test("theme: toggling flips the document `dark` class", async ({ page }) => {
    await page.goto("/ui/theme");
    await waitForHydration(page);
    const html = page.locator("html");
    const wasDark = (await html.getAttribute("class"))?.includes("dark") ?? false;
    await page.getByRole("button", { name: "Toggle dark mode" }).first().click();
    if (wasDark) await expect(html).not.toHaveClass(/dark/);
    else await expect(html).toHaveClass(/dark/);
  });

  test("inputs: a Switch flips aria-checked on click", async ({ page }) => {
    await page.goto("/ui/inputs");
    await waitForHydration(page);
    const sw = page.locator('[role="switch"]').first();
    const before = await sw.getAttribute("aria-checked");
    await sw.click();
    await expect.poll(() => sw.getAttribute("aria-checked")).not.toBe(before);
  });

  test("inputs: a Checkbox toggles its checked state", async ({ page }) => {
    await page.goto("/ui/inputs");
    await waitForHydration(page);
    const cb = page.locator('[role="checkbox"]').first();
    const before = await cb.getAttribute("aria-checked");
    await cb.click();
    await expect.poll(() => cb.getAttribute("aria-checked")).not.toBe(before);
  });

  test("overlays: a Dialog opens on its trigger and closes on Escape", async ({ page }) => {
    await page.goto("/ui/overlays");
    await waitForHydration(page);
    // Scope to the Dialog's own content, which is gated by <Show> (mounted only
    // while open) — unlike sibling overlays that keep content mounted off-screen.
    const content = page.locator('[data-name="DialogContent"]');
    await expect(content).toHaveCount(0);
    await page.locator('[aria-haspopup="dialog"]').first().click();
    await expect(content.first()).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(content).toHaveCount(0);
  });

  test("overlays: the command palette opens on Cmd/Ctrl+K", async ({ page }) => {
    await page.goto("/ui/overlays");
    await waitForHydration(page);
    await page.locator("body").click();
    await page.keyboard.press("ControlOrMeta+k");
    await expect(page.locator('[role="dialog"]:visible').first()).toBeVisible();
  });

  test("navigation: selecting a Tab activates its panel", async ({ page }) => {
    await page.goto("/ui/navigation");
    await waitForHydration(page);
    const second = page.locator('[role="tab"]').nth(1);
    await second.click();
    await expect(second).toHaveAttribute("aria-selected", "true");
  });

  test("feedback: a toast trigger raises a toast", async ({ page }) => {
    await page.goto("/ui/feedback");
    await waitForHydration(page);
    await page.getByRole("button", { name: "Success" }).first().click();
    await expect(page.locator('[data-name="Toast"]').first()).toBeVisible();
  });

  test("forms: typing into the OTP mirrors into the readout", async ({ page }) => {
    await page.goto("/ui/forms");
    await waitForHydration(page);
    await page.locator("[data-otp-slot]").first().click();
    await page.keyboard.type("123456");
    await expect(page.getByText(/Entered:\s*123456/)).toBeVisible();
  });

  test("dates: clicking a day marks it the current selection", async ({ page }) => {
    await page.goto("/ui/dates");
    await waitForHydration(page);
    const day = page
      .locator('[data-name="DatePickerCell"]:not([aria-disabled="true"])')
      .filter({ hasText: /^23$/ })
      .first();
    await day.click();
    await expect(day).toHaveAttribute("aria-current", "true");
  });

  test("layout: an Accordion expands on click", async ({ page }) => {
    await page.goto("/ui/layout");
    await waitForHydration(page);
    // The first item is default-open; target the second (closed) trigger.
    const trigger = page.locator('[data-name="AccordionTrigger"]').nth(1);
    await expect(trigger).toHaveAttribute("aria-expanded", "false");
    await trigger.click();
    await expect(trigger).toHaveAttribute("aria-expanded", "true");
  });

  test("hooks: the copy button reports copied state", async ({ page }) => {
    await page.goto("/ui/hooks");
    await waitForHydration(page);
    await page.getByRole("button", { name: /Copy/ }).first().click();
    await expect(page.getByText("Copied!").first()).toBeVisible();
  });
});
