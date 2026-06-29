import { authenticator } from "otplib";
import { expect, Page, test } from "@playwright/test";

// These exercise the live dev stack (Postgres + the running server) against the
// bootstrap admin seeded by the `dev` profile (config/app.toml [dev.auth]). They
// mutate that single account's MFA/password state, so they run serially and each
// restores the account to a password-only baseline.
const ADMIN_EMAIL = process.env.E2E_ADMIN_EMAIL ?? "admin@openworkspace.test";
const ADMIN_PASSWORD = process.env.E2E_ADMIN_PASSWORD ?? "devadminpassword";

test.describe.configure({ mode: "serial" });

// This suite mutates one shared account and uses a Chromium-only virtual
// authenticator, so it runs on a single browser to avoid cross-project races.
test.beforeEach(({ browserName }) => {
  test.skip(browserName !== "chromium", "stateful auth flow runs on Chromium only");
});

/** Navigate and wait for the wasm bundle to load + hydrate before interacting. */
async function gotoApp(page: Page, path: string) {
  await page.goto(path);
  await page.waitForLoadState("networkidle");
}

/** Activate a tab, retrying until it reports selected (survives hydration timing). */
async function openTab(page: Page, name: RegExp) {
  const tab = page.getByRole("tab", { name });
  await expect(async () => {
    await tab.click();
    await expect(tab).toHaveAttribute("aria-selected", "true", { timeout: 1000 });
  }).toPass({ timeout: 15000 });
}

async function fillPassword(page: Page, email: string, password: string) {
  await page.locator('input[type="email"]').fill(email);
  await page.locator('input[type="password"]').fill(password);
  await page.getByRole("button", { name: /^sign in$/i }).click();
}

/** Sign in with a password and wait until we leave the login page. */
async function signIn(page: Page, email = ADMIN_EMAIL, password = ADMIN_PASSWORD) {
  await gotoApp(page, "/login");
  await fillPassword(page, email, password);
  await expect(page).not.toHaveURL(/\/login/, { timeout: 20000 });
}

/** A page that only renders for an authenticated session proves we are signed in. */
async function expectSignedIn(page: Page) {
  await gotoApp(page, "/build");
  await expect(page.getByRole("heading", { name: /floor builder/i })).toBeVisible();
  await expect(page.getByRole("button", { name: /^sign in$/i })).toHaveCount(0);
}

test("password sign-in reaches an authenticated page", async ({ page }) => {
  await signIn(page);
  await expectSignedIn(page);
});

test("the dashboard user menu signs out", async ({ page }) => {
  await signIn(page);
  await gotoApp(page, "/dashboard");
  await page.getByText("shadcn", { exact: true }).first().click();
  await page.getByRole("menuitem", { name: /log out/i }).click();
  await expect(page).toHaveURL(/\/login/, { timeout: 20000 });
});

test("enrol TOTP, sign in with a code, then disable it", async ({ page }) => {
  await signIn(page);
  await gotoApp(page, "/account");
  await openTab(page, /two-factor/i);
  await page.getByRole("button", { name: /set up authenticator/i }).click();

  const secret = (await page.locator("code").first().innerText()).trim();
  await page.locator('input[data-input-otp="true"]').first().fill(authenticator.generate(secret));
  await page.getByRole("button", { name: /^confirm$/i }).click();
  await expect(page.getByText(/save your recovery codes/i)).toBeVisible();

  // A fresh sign-in now demands the second factor.
  await page.context().clearCookies();
  await gotoApp(page, "/login");
  await fillPassword(page, ADMIN_EMAIL, ADMIN_PASSWORD);
  await expect(page.getByText(/two-factor authentication/i)).toBeVisible();
  await page.locator('input[data-input-otp="true"]').first().fill(authenticator.generate(secret));
  await page.getByRole("button", { name: /^verify$/i }).click();
  await expect(page).not.toHaveURL(/\/login/, { timeout: 20000 });

  // Restore the password-only baseline.
  await gotoApp(page, "/account");
  await openTab(page, /two-factor/i);
  await page.getByRole("button", { name: /^disable$/i }).click();
  await page.getByRole("button", { name: /click again to disable/i }).click();
  await expect(page.getByRole("button", { name: /set up authenticator/i })).toBeVisible();
});

test("register a passkey and sign in passwordlessly", async ({ browserName, page }) => {
  test.skip(browserName !== "chromium", "the virtual authenticator is Chromium/CDP only");

  // A virtual authenticator with a resident key auto-satisfies the ceremony.
  const client = await page.context().newCDPSession(page);
  await client.send("WebAuthn.enable");
  await client.send("WebAuthn.addVirtualAuthenticator", {
    options: {
      protocol: "ctap2",
      transport: "internal",
      hasResidentKey: true,
      hasUserVerification: true,
      isUserVerified: true,
      automaticPresenceSimulation: true,
    },
  });

  await signIn(page);
  await gotoApp(page, "/account");
  await openTab(page, /passkeys/i);
  await page.getByRole("button", { name: /add a passkey/i }).click();
  await expect(page.getByText(/passkey added/i)).toBeVisible({ timeout: 20000 });

  // Drop the session cookie, then sign in with email + passkey (no password).
  await page.context().clearCookies();
  await gotoApp(page, "/login");
  await page.locator('input[type="email"]').fill(ADMIN_EMAIL);
  await page.getByRole("button", { name: /sign in with a passkey/i }).click();
  await expect(page).not.toHaveURL(/\/login/, { timeout: 20000 });
  await expectSignedIn(page);

  // Cleanup: remove every passkey (two clicks each = arm + confirm).
  await gotoApp(page, "/account");
  await openTab(page, /passkeys/i);
  const remove = page.getByRole("button", { name: /^remove$/i });
  while ((await remove.count()) > 0) {
    await remove.first().click(); // arm
    await remove.first().click(); // confirm
    await page.waitForTimeout(500);
  }
  await expect(page.getByText(/no passkeys yet/i)).toBeVisible({ timeout: 20000 });
});

test("change the password and change it back", async ({ page }) => {
  const temp = "devadminpassword-temp";
  await signIn(page);
  await gotoApp(page, "/account"); // Password tab is the default
  await page.locator("#current_pw").fill(ADMIN_PASSWORD);
  await page.locator("#new_pw").fill(temp);
  await page.locator("#confirm_pw").fill(temp);
  await page.getByRole("button", { name: /change password/i }).click();
  await expect(page.getByText(/your password has been changed/i)).toBeVisible({ timeout: 20000 });

  // The new password works; restore the original so re-runs stay deterministic.
  await page.context().clearCookies();
  await signIn(page, ADMIN_EMAIL, temp);
  await gotoApp(page, "/account");
  await page.locator("#current_pw").fill(temp);
  await page.locator("#new_pw").fill(ADMIN_PASSWORD);
  await page.locator("#confirm_pw").fill(ADMIN_PASSWORD);
  await page.getByRole("button", { name: /change password/i }).click();
  await expect(page.getByText(/your password has been changed/i)).toBeVisible({ timeout: 20000 });
});
