# 13 — GitHub Pages Website

This document describes the React‑Vite website, its routing, and known
discrepancies between website copy and actual app behavior.

## 1. Configuration

- **Vite base path:** `base: '/nixobdo-pdf'` (set in `website/vite.config.ts`).
- **GitHub Pages repo:** `borneelphukan.github.io/nixobdo-pdf`; the site is built
  and deployed from the `main` branch only.
- **Basename:** All routes are prefixed with `/nixobdo-pdf/`. For example, the
  about page is at `/nixobdo-pdf/about`, not `/about`.

## 2. Routes (configured in Vite + React Router)

| Route | Path |
|---|---|
| Home / docs index | `/nixobdo-pdf/` (or `/nixobdo-pdf/docs`) |
| Features | `/nixobdo-pdf/docs/guides/features` |
| Guides | `/nixobdo-pdf/docs/guides` |
| About | `/nixobdo-pdf/about` |

## 3. Known website bugs

### a) Home page link mismatch

- The home page links to `/docs/features/features`, but the configured route is
  `/nixobdo-pdf/docs/guides/features`. This mismatch means clicking the home
  "Features" link may produce a 404 or redirect depending on the exact GitHub
  Pages settings.

### b) No 404.html fallback

- The Vite/React build does **not** generate a `404.html` file. On GitHub Pages,
  deep‑link refreshes (e.g. navigating directly to
  `/nixobdo-pdf/docs/guides/features`) will result in a 404 if the server is not
  configured for client‑side routing. This is a known limitation; the site assumes
  all navigation goes through React Router.

### c) Aspirational vs actual feature copy

The website documents several features that **do not exist** in the current Rust
codebase, including:

- Drag‑and‑drop PDF opening
- Freehand drawing
- Sticky notes
- Thumbnail grid (the app has a thumbnail sidebar, but the website describes a
  grid layout)
- Tool menu locations that differ from the actual egui menu bar
- Any "cloud save" or "sync" functionality

These discrepancies exist because the website was written as aspirational/Marketing
copy rather than a faithful description of the shipped code. Future work should
either align the copy with actual behavior or remove the aspirational sections.

## 4. Build and deploy

- `cargo run -p nixobdo-pdf-website` (or equivalent) builds the React app with
  Vite using the `/nixobdo-pdf` base.
- The `website.yml` workflow runs on `main` pushes and PRs, lints the build, and
  triggers `deploy.yml` to push to GitHub Pages.
- Deployment is **only** from `main`; pushes to `dev` do NOT deploy, even
  though the build may run.