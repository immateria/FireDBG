# FireDBG Smoke Test (Minimal Workflow)
This is a small “does it basically work?” workflow you (or contributors) can run locally.

Assumptions:
- You have Rust installed (`rustc`, `cargo`).
- You are using the source-first installation from this repo.

## 1) Install from source
From the repo root:

```sh
./install.sh

# Optional: see what FireDBG thinks your environment looks like
firedbg doctor
```

## 2) Choose a target workspace
FireDBG operates on a Cargo workspace.

This repo includes a tiny workspace you can use immediately:

```sh
cd examples/smoke-workspace
firedbg list-target
```

You should see a binary target named `app`.

## 3) Record a run
Run the listed binary:

```sh
firedbg run app
```

Or specify an explicit output path:

```sh
firedbg run app --output ./firedbg/target/app.firedbg.ss
```

## 4) List runs + index

```sh
firedbg list-run

# Index the latest run (idx 1)
firedbg index 1
```

This should create a `.sqlite` file alongside the `.firedbg.ss` file.

## 5) Open in VS Code
If you have VS Code + the `code` CLI installed:

```sh
firedbg open 1
```

If `firedbg open` fails, run:

```sh
firedbg doctor
```

## 6) Cleanup
To remove FireDBG workspace artifacts:

```sh
firedbg clean
```

To remove cached CodeLLDB assets (installer cache):

```sh
# from the FireDBG.for.Rust repo root
./install.sh --clean-cache
```
