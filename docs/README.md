# TSI Documentation

This directory contains the active docs for the Telescope Scheduling Intelligence (TSI) dashboard and its Python + Rust backend.

The index below reflects the current repository state and avoids links to non-existent pages.

## Quick Links
- Setup Guide: [SETUP.md](./SETUP.md)
- Build Rust backend: `./build_rust.sh` (see script help with `--help`)
- Run dashboard: `./run_dashboard.sh`
- Scripts overview: `../scripts/README.md`

## What’s Here
- Streamlit dashboard in `src/tsi/`
- Rust backend crate `backend/` producing the `tsi_rust` Python module via PyO3/Maturin
- Helper scripts in `scripts/` for build, local runs, and utilities

## Getting Started
Start with the Setup Guide for environment, database, build, and run instructions: [SETUP.md](./SETUP.md)

For top-level project context, see the repository `README.md`.
