# infinity-website

The [Infinity Simulations](https://infinity-simulations.com) website — a Leptos
CSR single-page app compiled to WebAssembly and served from Cloudflare Pages.

## Prerequisites

- **Rust 1.96+** with the wasm target: `rustup target add wasm32-unknown-unknown`
- **[Trunk](https://trunkrs.dev)** 0.21+: `cargo install --locked trunk`
- **[kivro](https://kivro.xyz)** — supplies build-time config (see below)
- **Node** — only for deploying (`npx wrangler`)

Trunk downloads its own `tailwindcss` and `wasm-opt` on first run; nothing else
to install.

### Sibling checkout

The component library is a path dependency, so it has to sit next to this repo:

```
github/
├── infinity-website/     # this repo
└── infinity-ui-wasm/     # https://github.com/infinity-MSFS/infinity-ui
```

`Cargo.toml`, `tailwind.config.js`, and `input.css` all reference
`../infinity-ui-wasm`. Without it, nothing builds.

## Configuration

Endpoints and Auth0 client config are read from the environment at compile time
and baked into the wasm. They're declared in `.kivro.toml` and stored with
[kivro](https://kivro.xyz):

| Variable | Purpose |
| --- | --- |
| `CDN_WORKER_URL` | CDN worker; falls back to the public production URL |
| `SUPPORT_API_URL` | Support ticket backend |
| `AUTH0_DOMAIN` | Auth0 tenant |
| `AUTH0_CLIENT_ID` | Auth0 SPA client |
| `AUTH0_AUDIENCE` | Auth0 API audience |

None of these are secrets — a browser can read every one out of the shipped
bundle. They live in kivro so the values stay out of git and so each
environment can point at its own backends.

Set them once:

```sh
kivro set AUTH0_DOMAIN        # repeat for each variable above
kivro status                  # confirms nothing is missing
```

Then prefix any build with `kivro run --`, which injects them for the duration
of that command. Missing variables don't fail the build — you get a
`cargo:warning` and a bundle that runs signed out with dead endpoints.

## Develop

```sh
kivro run -- trunk serve
```

Serves on <http://localhost:3035> with hot reload, watching this crate and the
sibling `infinity-ui` sources.

## Build

```sh
kivro run -- trunk build --release
```

Output lands in `dist/`. Always use `--release`: the debug wasm is ~22 MB,
versus a few MB optimized, and Cloudflare rejects files over 25 MiB.

## Deploy

```powershell
./scripts/deploy.ps1                    # preview of branch "main"
./scripts/deploy.ps1 -Branch my-feature # preview of another branch
./scripts/deploy.ps1 -Production        # production (live domains)
./scripts/deploy.ps1 -Env prod          # build against a different kivro env
```

The script checks config, builds a release bundle, verifies the artifacts, and
Direct Uploads `dist/` to Cloudflare Pages via `wrangler`. We build locally
rather than on Cloudflare because their build image ships no Rust toolchain and
a cold release build overruns their time budget.

`public/_redirects` rewrites every path to `index.html` so the client-side
router handles deep links; `public/_headers` pins the content-hashed assets and
keeps the shell uncached. Both are copied into `dist/` by Trunk — the deploy
script refuses to upload without them.

When adding a new domain, add it to the Auth0 application's **Allowed Callback
URLs**, **Allowed Logout URLs**, and **Allowed Web Origins**, or sign-in breaks
on that host.

## License

MIT — see [LICENSE](LICENSE). The bundled `auth0-spa-js` build in
`public/vendor/` is MIT licensed by Auth0; its notice is alongside it.
