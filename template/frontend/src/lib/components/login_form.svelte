<script>
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { FieldGroup, Field, FieldLabel } from "$lib/components/ui/field/index.js";

  const {
    id,
    whoami = "/api/whoami",
    loginAction = "/api/login",
    logoutAction = "/api/logout",
  } = $props();

  let email = $state("");
  let password = $state("");

  let csrfToken = $state("");
  let isLoggedIn = $state(false);
  let userId = $state(null);

  let loading = $state(false);
  let errorMsg = $state("");

  // ✅ gate initial render on this
  let sessionLoaded = $state(false);

  function applyWhoami(json) {
    const session = json?.data?.session;
    csrfToken = typeof session?.csrf_token === "string" ? session.csrf_token : "";
    isLoggedIn = !!session?.is_logged_in;
    userId = session?.external_user_id ?? null;
  }

  async function fetchWhoami() {
    try {
      const res = await fetch(whoami, { credentials: "include" });
      const json = await res.json().catch(() => null);
      applyWhoami(json);
      return { res, json };
    } finally {
      // ✅ ensures we stop showing the spinner even if whoami errors
      sessionLoaded = true;
    }
  }

  onMount(fetchWhoami);

  async function ensureCsrf() {
    if (csrfToken) return true;
    await fetchWhoami();
    return !!csrfToken;
  }

  async function postJson(url, bodyObj) {
    return fetch(url, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "X-CSRF-Token": csrfToken,
      },
      body: JSON.stringify(bodyObj ?? {}),
    });
  }

  async function submitLogin(event) {
    event.preventDefault();
    loading = true;
    errorMsg = "";

    if (!(await ensureCsrf())) {
      loading = false;
      errorMsg = "Could not retrieve CSRF token. Please refresh and try again.";
      return;
    }

    let res = await postJson(loginAction, { email, password });
    let json = await res.json().catch(() => null);

    if (
      res.status === 401 &&
      (json?.error?.code === "csrf_invalid" || json?.error?.code === "csrf_missing")
    ) {
      await fetchWhoami();
      res = await postJson(loginAction, { email, password });
      json = await res.json().catch(() => null);
    }

    if (!res.ok || json?.error) {
      errorMsg = json?.error?.message ?? json?.error ?? `Login failed (HTTP ${res.status})`;
      loading = false;
      return;
    }

    await fetchWhoami();
    password = "";
    loading = false;
  }

  async function submitLogout() {
    loading = true;
    errorMsg = "";

    if (!(await ensureCsrf())) {
      loading = false;
      errorMsg = "Could not retrieve CSRF token. Please refresh and try again.";
      return;
    }

    let res = await postJson(logoutAction, null);
    let json = await res.json().catch(() => null);

    if (
      res.status === 401 &&
      (json?.error?.code === "csrf_invalid" || json?.error?.code === "csrf_missing")
    ) {
      await fetchWhoami();
      res = await postJson(logoutAction, null);
      json = await res.json().catch(() => null);
    }

    if (!res.ok || json?.error) {
      errorMsg = json?.error?.message ?? json?.error ?? `Logout failed (HTTP ${res.status})`;
      loading = false;
      return;
    }

    // ✅ token is refreshed on logout; grab the new one
    await fetchWhoami();
    loading = false;
  }
</script>

{#if !sessionLoaded}
  <!-- blank-ish page + centered loader -->
  <div class="mx-auto flex w-full max-w-sm items-center justify-center py-10">
    <div
      class="h-6 w-6 animate-spin rounded-full border-2 border-current border-t-transparent"
      aria-label="Loading"
    ></div>
  </div>
{:else}
  <Card.Root class="mx-auto w-full max-w-sm">
    <Card.Header>
      {#if isLoggedIn}
        <Card.Title class="text-2xl">You’re signed in</Card.Title>
        <Card.Description>
          Signed in as <span class="font-mono">{userId ?? "(unknown)"}</span>
        </Card.Description>
      {:else}
        <Card.Title class="text-2xl">Sign in</Card.Title>
        <Card.Description>Enter your account credentials:</Card.Description>
      {/if}
    </Card.Header>

    <Card.Content>
      {#if errorMsg}
        <p class="mb-3 text-sm text-red-600">{errorMsg}</p>
      {/if}

      {#if isLoggedIn}
        <Button class="w-full" onclick={submitLogout} disabled={loading}>
          {loading ? "Signing out…" : "Logout"}
        </Button>
      {:else}
        <form onsubmit={submitLogin}>
          <FieldGroup>
            <Field>
              <FieldLabel for={"email-" + id}>Email</FieldLabel>
              <Input
                id={"email-" + id}
                type="email"
                placeholder="m@example.com"
                autocomplete="email"
                required
                bind:value={email}
              />
            </Field>

            <Field>
              <FieldLabel for={"password-" + id}>Password</FieldLabel>
              <Input
                id={"password-" + id}
                type="password"
                autocomplete="current-password"
                required
                bind:value={password}
              />
            </Field>

            <Field>
              <Button type="submit" class="w-full" disabled={loading}>
                {loading ? "Logging in…" : "Login"}
              </Button>
            </Field>
          </FieldGroup>
        </form>
      {/if}
    </Card.Content>
  </Card.Root>
{/if}
