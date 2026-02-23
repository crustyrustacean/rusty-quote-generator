# Research Report: `rusty-quote-generator`

**Author:** Jeffery D. Mitchell  
**Date Analyzed:** 2026-02-22  
**Repository:** https://github.com/crustyrustacean/actix-web-starter.git

---

## Executive Summary

`rusty-quote-generator` is a full-stack Rust web application built as a learning project — a Rust/Yew conversion of the "Quote Generator" project from Zero to Mastery's *20 JavaScript Projects* course. It uses a **Cargo workspace** to house two separate crates: a **Yew** WebAssembly frontend and an **Actix Web** backend. The backend serves the compiled frontend as static files, forming a classic single-binary deployment model. The project is mature enough to include Docker multi-stage builds, structured tracing/logging, environment-based configuration, and integration tests.

---

## Project Structure

```
rusty-quote-generator/
├── Cargo.toml              # Workspace root
├── Dockerfile              # Multi-stage Docker build
├── justfile                # Task runner (just)
├── backend/
│   ├── Cargo.toml
│   ├── configuration/      # YAML config files (base, local, production)
│   ├── src/
│   │   ├── bin/main.rs     # Binary entry point
│   │   ├── lib.rs          # Library root
│   │   ├── configuration.rs
│   │   ├── startup.rs
│   │   ├── routes.rs
│   │   ├── routes/health_check.rs
│   │   └── telemetry.rs
│   └── tests/api/          # Integration tests
│       ├── main.rs
│       ├── helpers.rs
│       └── health_check.rs
└── frontend/
    ├── Cargo.toml
    ├── Cargo.lock          # Frontend-specific lockfile
    ├── Trunk.toml          # Trunk build config
    ├── index.html          # Trunk entry point
    ├── styles.css          # Full CSS stylesheet
    └── src/
        ├── bin/main.rs     # WASM entry point
        ├── lib.rs
        ├── app.rs
        ├── domain.rs
        ├── components.rs
        ├── components/quote_card.rs
        ├── views.rs
        └── views/home_view.rs
```

---

## Workspace Configuration (`Cargo.toml`)

The workspace uses **Cargo resolver version 3** (Rust edition 2024). It defines shared package metadata (`[workspace.package]`) that member crates inherit via `.workspace = true`, keeping version numbers and author info DRY. Both `backend` and `frontend` are members; `backend` is the default build target.

Release profiles are heavily optimized:
- `lto = true` — link-time optimization for smaller binaries
- `codegen-units = 1` — maximum optimization, slower compile
- `strip = true` — removes debug symbols from release binary
- `panic = "abort"` — avoids unwinding overhead in production

Dev profiles enable debug info normally, but dependency packages get `opt-level = 2` to keep dev builds reasonably fast despite unoptimized application code.

---

## Backend

### Crate Layout

The backend exposes both a **library** (`rusty_quote_generator_server`) and a **binary** (`rqg-server`). The library encapsulates all logic modules — configuration, startup, routes, and telemetry — and re-exports them from `lib.rs`. The binary (`bin/main.rs`) is a thin entry point that wires everything together. This library/binary split is a pattern taken directly from *Zero to Production in Rust* and makes the codebase testable without spinning up the binary directly.

### Configuration (`configuration.rs`)

Configuration is layered:
1. `configuration/base.yaml` — shared defaults (port 8000, host `0.0.0.0`)
2. `configuration/{environment}.yaml` — overrides per environment (local or production)
3. Environment variables — prefixed with `APP_`, using `__` as separator (e.g., `APP_APPLICATION__PORT=9000`)

The `Environment` enum (`Local` | `Production`) is parsed from the `APP_ENVIRONMENT` env var, defaulting to `"local"`. The `config` crate assembles these layers. `serde-aux` provides `deserialize_number_from_string` so the port can be set via string env vars without type errors. The `ApplicationSettings` struct requires a `base_url` field; notably, **`base.yaml` does not define `base_url`**, meaning that `local.yaml` must supply it, or the config read will fail — a subtle initialization requirement.

### Startup (`startup.rs`)

The `Application` struct wraps an Actix `Server` instance and its port. `Application::build()` reads the configuration, binds a `TcpListener`, and calls `run()`. Using a `TcpListener` (rather than passing the address string directly to `HttpServer`) enables port `0` to be used in tests, letting the OS assign a random available port — crucial for test isolation.

The Actix `App` is configured with two services:
- `GET /health_check` — mapped to the `health_check` handler
- `Files::new("/", "../public")` — serves the compiled Yew WASM app from the `../public` directory (relative to where the binary runs), with `index_file("index.html")` and UTF-8 preference

The static file serving is handled by `actix-files`. The relative path `../public` works because the backend binary is expected to run from the `backend/` directory in dev, and from `/app/backend` in Docker (where `/app/public` is one level up). The `ApplicationBaseUrl` newtype wraps the base URL and is registered as Actix `Data` for potential injection into handlers.

### Routes (`routes/health_check.rs`)

A single route exists: `GET /health_check`. It returns `HttpResponse::Ok()` with an empty body. This is used for liveness probes (Docker/fly.io) and as the subject of the integration test. The empty body (content length 0) is a deliberate choice, verified in the test.

### Telemetry (`telemetry.rs`)

The backend uses the **tracing** ecosystem:
- `tracing-subscriber` with `EnvFilter` for log-level control via `RUST_LOG`
- `tracing-bunyan-formatter` for structured JSON logging (Bunyan format)
- `tracing-log` bridges the `log` crate into `tracing`
- `tracing-actix-web` (registered as a dependency but not yet wired into the `App` as middleware — a notable omission, likely a TODO)

`get_subscriber` composes these layers and returns a type-erased `impl Subscriber + Sync + Send`. `init_subscriber` sets it as the global default. The `spawn_blocking_with_tracing` helper propagates the current tracing span into blocking tasks spawned via `actix_web::rt::task::spawn_blocking` — important for maintaining trace context across async/sync boundaries.

### Integration Tests (`tests/api/`)

Tests live in `tests/api/`, structured as a multi-file integration test binary (`main.rs` declares modules; `helpers.rs` and `health_check.rs` contain the logic).

`spawn_app()` in `helpers.rs`:
- Uses `LazyLock` for one-time tracing initialization (avoids duplicate subscriber errors)
- Clones and mutates configuration, setting `port = 0` for OS-assigned ports
- Spawns the application server as a background task via `tokio::spawn`
- Builds a `reqwest::Client` with redirect-following disabled
- Returns a `TestApp` with `address` and `port`

The `TEST_LOG` environment variable switches tracing output from `/dev/null` (`io::sink`) to stdout, enabling debug output during test development.

The `health_check_works` test verifies both status code (success) and content length (0 bytes), which matches the handler's `HttpResponse::Ok()` with no body.

---

## Frontend

### Crate Layout

The frontend crate (`rusty-quote-generator-client`) also splits into a library and binary. The binary (`bin/main.rs`) initializes the WASM logger, sets the panic hook for browser-friendly error messages, and renders the `App` component via `yew::Renderer`. The library defines all Yew components and domain types.

### Module Organization

```
app.rs          — Root App component; composes HomeView + QuoteCard
domain.rs       — Quote struct (data model)
components/
  quote_card.rs — Core interactive component
views/
  home_view.rs  — Layout wrapper with children prop
```

### Domain Model (`domain.rs`)

```rust
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Quote {
    pub text: AttrValue,
    pub author: AttrValue,
    pub tag: Option<AttrValue>,
}
```

`AttrValue` (from Yew) is used instead of `String` for text fields. This is idiomatic in Yew because `AttrValue` is cheaply cloneable (it's an `Rc<str>` under the hood) and can be passed directly into HTML attributes without string conversion overhead. `tag` is optional because not all quotes in the API have tags.

### `QuoteCard` Component (`components/quote_card.rs`)

This is the heart of the application. It uses `#[component]` (the hook-based Yew macro) and manages two state handles:
- `quotes: UseStateHandle<Vec<Quote>>` — the full fetched list
- `selected_quote: UseStateHandle<Option<Quote>>` — the currently displayed quote

**Data Fetching on Mount:**
`use_effect_with((), ...)` runs once on component mount (the `()` dependency means "no reactive dependency, run once"). Inside, `wasm_bindgen_futures::spawn_local` launches an async block that uses `gloo_net::http::Request::get()` to fetch all quotes from the external Jacinto Design API (`https://jacintodesign.github.io/quotes-api/data/quotes.json`). The response is deserialized as `Vec<Quote>` and stored in state. **Errors are handled with `unwrap()`** — a valid choice for a learning project, but would be hardened in production.

**Random Quote Selection:**
The `on_new_quote` callback clones both state handles, checks that quotes are non-empty, then uses `js_sys::Math::random()` to pick a random index. This directly calls the browser's `Math.random()` via WASM bindings — the idiomatic approach in a WASM context (Rust's `rand` crate requires WASM-specific configuration and extra dependencies).

**Tweet Integration:**
`on_tweet` constructs an X (Twitter) intent URL with the quote text and author URL-encoded inline (using Rust's `format!` macro, not proper URL encoding — which could cause issues with special characters). It opens the URL in a new tab via `web_sys::window().unwrap().open_with_url_and_target()`.

**Rendering:**
The component uses a match on `*selected_quote` to either display the actual quote or a placeholder. The HTML structure:
```
<main class="quote-container">
  <section>
    <article class="quote-text">  — quote text with Font Awesome left-quote icon
    <article class="quote-author"> — author name
    <article class="button-container"> — Tweet button + New Quote button
  </section>
</main>
```

### `HomeView` Component (`views/home_view.rs`)

A minimal layout wrapper. It accepts `Children` as a prop and renders them inside a `<div>`. This follows Yew's composition pattern — `App` passes `<QuoteCard />` as children, letting `HomeView` control layout structure independently of content. Currently trivial, but provides an extension point for future layout changes (headers, footers, navigation).

### `App` Component (`app.rs`)

Composes `HomeView` with `QuoteCard` as its child. Simple and declarative.

### Build Configuration

**`Trunk.toml`:** Sets `dist = "../public"`, pointing Trunk's output to the workspace-level `public/` directory. The backend's `Files::new("/", "../public")` then serves from this same location, creating a clean handoff between the two tools.

**`index.html`:** Uses Trunk's data-trunk attributes:
- `<link data-trunk rel="css" href="styles.css" />` — inlines/processes the CSS
- `<Link data-trunk rel="copy-dir" href="assets" />` — copies assets directory to dist
- `<link data-trunk rel="icon" ...>` — handles favicon
- Font Awesome 5.10.2 from CDN for icons

### Styling (`styles.css`)

The CSS is a faithful port of the original JavaScript project's styles:
- Google Fonts Montserrat import
- SVG pattern background embedded as a data URI (avoiding an external network request)
- Flexbox centering on body (`display: flex; align-items: center; justify-content: center`)
- `.quote-container` has `max-width: 900px` with semi-transparent white background and a box-shadow
- Responsive breakpoint at 1000px adjusts font sizes and container margins
- Button hover (`filter: brightness(110%)`) and active (`transform: translate(0, 0.3rem)`) states add tactile feel
- `.long-quote` class exists in CSS but is **not yet applied dynamically** — a TODO for conditionally shrinking font size for long quotes (the JS original did this)
- Loader spinner styles (`@keyframes spin`) are defined but **not used** — another carryover from the original or planned feature

---

## Docker Multi-Stage Build (`Dockerfile`)

The Dockerfile uses five stages for maximum cache efficiency and minimal image size:

1. **`chef`** — Installs `cargo-chef` on the Rust base image
2. **`planner`** — Copies source and runs `cargo chef prepare` to generate `recipe.json` (a dependency manifest)
3. **`frontend-builder`** — Installs `trunk` and `wasm32-unknown-unknown` target, builds the Yew frontend to WASM
4. **`backend-builder`** — Uses `cargo chef cook` to pre-build dependencies (cached layer), then builds the backend binary. Notably references `rust:1.93.1` in `chef` but `rust:1.93` in `frontend-builder` — a minor inconsistency
5. **`runtime`** — `debian:bookworm-slim` with only `ca-certificates` installed; copies the server binary, public assets, and configuration. Sets `APP_ENVIRONMENT=production` and exposes port 8080

The runtime image doesn't run as a non-root user — a security hardening opportunity. The backend binary is named `rqg-server` in the Dockerfile copy step as `rusty-quote-generator-server`, which would need to match the actual binary output name (the binary name is `rqg-server` per `[[bin]] name` in `backend/Cargo.toml`).

---

## `justfile` (Task Runner)

Two recipes:
- `just dev` — runs `trunk serve --open` from the frontend directory (hot-reload dev server)
- `just build` — runs `trunk build --release` (production frontend build)

The backend is not included in the `dev` recipe, which means local development currently requires running the backend separately (or the frontend fetches directly from the external API, bypassing the backend entirely). This reflects the current architecture where the backend is only needed for serving static files in production — in dev, Trunk's built-in dev server handles everything.

---

## Key Architectural Observations

**Separation of concerns is well-established.** Domain types, components, and views are in distinct modules. The backend's library/binary split, configuration layering, and telemetry setup demonstrate patterns from *Zero to Production in Rust*.

**The backend is currently a static file server.** The only API it exposes is `health_check`. All quote-fetching logic runs entirely in the browser (WASM calls the external Jacinto API directly). This means the backend adds deployment complexity without adding API value yet — it's clearly scaffolding for future backend API routes.

**Error handling is deliberately naive.** `unwrap()` is used throughout the frontend's async fetch logic. This is appropriate for a learning project but would be replaced with proper error state in production.

**The `base_url` configuration gap.** `base.yaml` doesn't define `base_url`, so running without either `local.yaml` or `production.yaml` (or an `APP_APPLICATION__BASE_URL` env var) will panic at startup. This is a minor brittleness inherited from the *Zero to Production* template.

**The `tracing-actix-web` middleware is included as a dependency but not registered** as middleware in the `App` builder in `startup.rs`. Adding `.wrap(TracingLogger::default())` would enable per-request structured logging.

**Frontend has no routing.** The app is single-view; there's no `yew-router` dependency. The `HomeView`/`App` separation provides the structure to add routing later without restructuring the component tree.

**Tweet URL construction lacks URL encoding.** Quotes with special characters (`&`, `#`, `?`, etc.) in `display_text` or `display_author` could malform the tweet intent URL. The JS original likely had the same issue, or used `encodeURIComponent`.

---

## Dependency Inventory

### Backend Dependencies
| Crate | Version | Purpose |
|---|---|---|
| `actix-web` | 4.13.0 | HTTP server framework |
| `actix-files` | 0.6.10 | Static file serving |
| `anyhow` | 1.0.102 | Ergonomic error handling |
| `config` | 0.15.19 | YAML configuration loading |
| `reqwest` | 0.13.2 | HTTP client (used in tests) |
| `serde` | 1.0.228 | Serialization/deserialization |
| `serde-aux` | 4.7.0 | `deserialize_number_from_string` |
| `tokio` | 1 | Async runtime |
| `tracing` | 0.1.19 | Structured logging/spans |
| `tracing-actix-web` | 0.7 | Actix request tracing middleware |
| `tracing-bunyan-formatter` | 0.3.1 | JSON Bunyan log format |
| `tracing-log` | 0.2.0 | Bridge `log` → `tracing` |
| `tracing-subscriber` | 0.3 | Subscriber composition |
| `log` | 0.4 | Log facade |

### Frontend Dependencies
| Crate | Version | Purpose |
|---|---|---|
| `yew` | 0.22.0 | WASM UI framework |
| `gloo-net` | 0.6.0 | Browser HTTP client |
| `wasm-bindgen` | 0.2.108 | Rust↔JS bindings |
| `wasm-bindgen-futures` | 0.4.58 | Async in WASM |
| `js-sys` | 0.3.88 | JS standard library bindings |
| `web-sys` | 0.3 | Browser Web API bindings |
| `serde` | 1.0.228 | Deserialize JSON quotes |
| `wasm-logger` | 0.2.0 | Log to browser console |
| `console_error_panic_hook` | 0.1.7 | Readable WASM panics |
| `log` | 0.4.29 | Log facade |

---

## Potential Next Steps / Observations for the Author

- **Wire up `tracing-actix-web`**: Add `.wrap(TracingLogger::default())` to the `App` in `startup.rs` to get structured per-request logs. FIXED
- **Add a backend quotes API**: Move quote fetching to a backend endpoint (e.g., `GET /api/quotes`) to gain experience with Actix route handlers, `reqwest` server-side HTTP calls, response caching, and JSON serialization.
- **Apply `.long-quote` dynamically**: In `QuoteCard`, check `display_text.len()` and conditionally add `"long-quote"` to the class list — the CSS is already there waiting for it.
- **URL-encode tweet text**: Use a crate like `urlencoding` or manually percent-encode the tweet URL components.
- **Add a loading state**: Show the spinner (CSS already defined) while quotes are being fetched — currently the "New Quote" button is silently inoperative until the fetch completes.
- **Harden error handling**: Replace `unwrap()` calls in the fetch logic with proper `UseStateHandle<Option<String>>` error state rendered in the UI.
- **Docker non-root user**: Add `RUN useradd -r appuser && chown appuser /app` and `USER appuser` in the runtime stage.
- **Fix `base_url` requirement**: Either add `base_url` to `base.yaml` or make `ApplicationSettings.base_url` an `Option<String>` to avoid startup panics in minimal environments.
