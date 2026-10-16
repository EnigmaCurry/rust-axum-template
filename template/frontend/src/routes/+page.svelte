<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Card from "$lib/components/ui/card/index.js";

  // The signed-in shell: this is where your app goes. It exists so that
  // /login can send an authenticated user somewhere, and so logout stays
  // reachable once /login stops rendering it.
  let checking = $state(true);
  let username = $state("");
  let csrfToken = $state("");
  let errorMsg = $state("");

  async function loadSession() {
    try {
      const res = await fetch("/api/whoami", { credentials: "include" });
      const json = await res.json().catch(() => null);
      const session = json?.data?.session;

      if (!session?.is_logged_in) {
        goto("/login", { replaceState: true });
        return;
      }

      username = typeof session.username === "string" ? session.username : "";
      csrfToken = typeof session.csrf_token === "string" ? session.csrf_token : "";
    } catch (e) {
      errorMsg = `Could not load your session. ${String(e)}`;
    } finally {
      checking = false;
    }
  }

  async function logout() {
    try {
      await fetch("/api/logout", {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json", "X-CSRF-Token": csrfToken },
        body: "{}",
      });
    } catch {
      // Clearing the session server-side is best effort; either way, leave.
    }
    goto("/login", { replaceState: true });
  }

  onMount(loadSession);
</script>

<div class="flex h-screen w-full items-center justify-center px-4">
  {#if checking}
    <p class="text-sm text-muted-foreground">Checking your session…</p>
  {:else}
    <Card.Root class="mx-auto w-full max-w-sm">
      <Card.Header>
        <Card.Title class="text-2xl">Signed in</Card.Title>
        <Card.Description>
          Signed in as <span class="font-mono">{username || "(unknown)"}</span>
        </Card.Description>
      </Card.Header>
      <Card.Content>
        {#if errorMsg}
          <p class="mb-3 text-sm text-red-600">{errorMsg}</p>
        {/if}
        <Button class="w-full" variant="secondary" onclick={logout}>Logout</Button>
      </Card.Content>
    </Card.Root>
  {/if}
</div>
