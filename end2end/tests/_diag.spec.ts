import { test } from "@playwright/test";

const PAGES = [
  "/ui/inputs",
  "/ui/overlays",
  "/ui/theme",
  "/ui/feedback",
  "/ui/forms",
  "/ui/hooks",
  "/ui/navigation",
];

for (const route of PAGES) {
  test(`diag ${route}`, async ({ page }, info) => {
    const msgs: string[] = [];
    page.on("console", (m) => msgs.push(`[${m.type()}] ${m.text()}`));
    page.on("pageerror", (e) => msgs.push(`[pageerror] ${e.message}`));
    await page.goto(route, { waitUntil: "networkidle" });
    await page.waitForTimeout(800);
    const facts = await page.evaluate(() => ({
      switches: document.querySelectorAll('[role="switch"]').length,
      dialogs: document.querySelectorAll('[role="dialog"]').length,
      buttons: document.querySelectorAll("button").length,
      htmlClass: document.documentElement.className,
      // does the wasm hydration island marker exist?
      hasHydrationIslands: !!document.querySelector("leptos-island, [data-hk]"),
      firstSwitchChecked: document
        .querySelector('[role="switch"]')
        ?.getAttribute("aria-checked"),
    }));
    const hydrationMsgs = msgs.filter((m) =>
      /hydrat|mismatch|panic|expected|not found|reconcil|island|debug_warn/i.test(m),
    );
    console.log(`\n===DIAG ${route}===`);
    console.log("FACTS " + JSON.stringify(facts));
    console.log("HYDRATION_MSGS " + JSON.stringify(hydrationMsgs.slice(0, 12)));
    console.log("ALL_MSG_COUNT " + msgs.length);
    console.log("FIRST_10_MSGS " + JSON.stringify(msgs.slice(0, 10)));
    await page.screenshot({
      path: `/tmp/diag-${route.replace(/\//g, "_")}.png`,
      fullPage: false,
    });
    info.skip(false);
  });
}
