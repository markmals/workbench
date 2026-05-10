---
name: web-verification
description: Use to drive the web app via the Chrome DevTools MCP for visual verification, UI debugging, and behavioral checks. Trigger when verifying web UI changes, screenshotting an app state, simulating interaction, or inspecting console/network in a tight verify-iterate loop. Analog of `ios-simulator-control` and `android-emulator-control` for the web platform.
---

# Web Verification

Drive the web app from Claude Code via the Chrome DevTools MCP. Tight verify-iterate loops: change code → reload → snapshot → assert → fix → repeat.

The MCP is already configured in [.mcp.json](.mcp.json) to use **Chromium** (not Chrome). Tools appear as `mcp__plugin_chrome-devtools-mcp_chrome-devtools__*` or similar.

## Prerequisites

- `mise run -C apps/web dev` running (default `http://localhost:5173/` for Vite-based projects; check the actual port in the dev server output).
- `mise run -C services/convex dev` running in parallel if the page touches Convex data.
- Chromium installed at the path configured in [.mcp.json](.mcp.json) (default macOS: `/Applications/Chromium.app/Contents/MacOS/Chromium`). If yours is elsewhere, edit the executablePath.

## Open a page

```
mcp__plugin_chrome-devtools-mcp_chrome-devtools__new_page
  url: "http://localhost:5173/"
```

For an existing tab:

```
mcp__plugin_chrome-devtools-mcp_chrome-devtools__navigate_page
  type: "url"
  url: "http://localhost:5173/items"
```

To reload (e.g. after a code change that didn't HMR-update):

```
mcp__plugin_chrome-devtools-mcp_chrome-devtools__navigate_page
  type: "reload"
  ignoreCache: true
```

## Screenshot

```
mcp__plugin_chrome-devtools-mcp_chrome-devtools__take_screenshot
  filePath: "<absolute path inside workspace>"   # e.g. apps/web/.tmp/screenshots/items-list.png
  fullPage: true                                  # or false for just the viewport
```

**Important:** screenshots can only be saved inside workspace roots. `/tmp/` will be rejected. Put them under a gitignored path inside the repo — e.g. `apps/web/.tmp/screenshots/` or `docs/.vitepress/cache/` (already gitignored).

After saving, use `Read` on the screenshot path to view it.

Take screenshots aggressively when verifying visual changes. Name them descriptively (`items-list-empty.png`, `items-list-populated.png`, `item-create-error.png`) so before/after comparison is easy.

## Inspect what's on screen

Take an accessibility snapshot — text representation of the DOM with stable element UIDs:

```
mcp__plugin_chrome-devtools-mcp_chrome-devtools__take_snapshot
```

Use the returned UIDs to drive interactions or assert structure. Prefer snapshots over screenshots when you're checking _content_ rather than _visuals_.

## Interact

```
mcp__plugin_chrome-devtools-mcp_chrome-devtools__click      uid: <from snapshot>
mcp__plugin_chrome-devtools-mcp_chrome-devtools__fill       uid: <from snapshot>  value: "Jane Doe"
mcp__plugin_chrome-devtools-mcp_chrome-devtools__hover      uid: <from snapshot>
mcp__plugin_chrome-devtools-mcp_chrome-devtools__press_key  key: "Escape"
mcp__plugin_chrome-devtools-mcp_chrome-devtools__type_text  text: "Hello"
```

For full-form input use `fill_form` with multiple UIDs and values.

## Read console messages

```
mcp__plugin_chrome-devtools-mcp_chrome-devtools__list_console_messages
  types: ["error", "warn"]   # filter to errors/warnings only when triaging
```

After a change, check the console **first** before deciding the change worked. Silent JS errors are a common cause of "looks blank but no obvious problem".

## Inspect network

```
mcp__plugin_chrome-devtools-mcp_chrome-devtools__list_network_requests
  resourceTypes: ["document", "fetch", "xhr"]
```

Drill into a specific request:

```
mcp__plugin_chrome-devtools-mcp_chrome-devtools__get_network_request
  reqid: <number from list>
```

Useful for catching unexpected requests (someone removed a query but the cache is stale, or a Convex call is firing twice).

## Evaluate JavaScript in the page

The escape hatch for everything else:

```
mcp__plugin_chrome-devtools-mcp_chrome-devtools__evaluate_script
  function: |
    () => ({
      title: document.title,
      hasItemList: !!document.querySelector('[data-testid="items-list"]'),
      itemCount: document.querySelectorAll('[data-testid="item-row"]').length,
    })
```

Use this to:

- Confirm Vue/React mounted (`document.getElementById('app').children.length > 0`)
- Read computed styles when a visual check is ambiguous
- Inspect window globals or page data when something behaves unexpectedly

Keep returned values JSON-serializable.

## Verify-iterate loop pattern

```
1. Make a code change in apps/web/.
2. The dev server should hot-reload automatically. If you suspect it didn't:
   - navigate_page  type: "reload"  ignoreCache: true
3. Drive the UI to the state you want to verify:
   - navigate_page → click → fill → … (one action per step is fine).
4. Wait for things to settle if the action is async:
   - wait_for       selector or condition
5. Take a screenshot OR a snapshot OR read console messages.
6. Decide: did the change work? If yes, move on. If no, repeat with a fix.
```

If you find yourself running the same sequence three times, define a helper `mise run -C apps/web verify-<feature>` task to script it.

## Common gotchas

- **404 for `/favicon.ico`** is harmless — the browser's default request. Our favicon is at `/favicon.svg`.
- **`Content-Type: text/html`** on what should be a static asset means VitePress / Vite is serving its SPA shell as a fallback because the URL didn't match a known asset. Check your `public/` resolution.
- **Empty `#app`** = Vue/React didn't mount. Almost always a runtime error in the entry script — check `list_console_messages` without filters.
- **Stale data after a Convex mutation** = the query cache wasn't invalidated. Check the mutation's `optimisticUpdate` / cache config in your view model.

## When NOT to use this skill

- Pure unit tests of view models or selectors — those run via `mise run -C apps/web test`, no browser needed.
- One-off lookups of page state that aren't tied to a code change.
- Anything that has a faster path through Vitest + jsdom.

## Related skills

- `web-development` — how to _write_ the code being verified.
- `implementing-a-spec` — the workflow this verification supports.
- `test-driven-development` — verification at the unit level (where most coverage lives).
- `verification-before-completion` — never claim a visual change works without doing this loop first.
