# Docker build: fast and minimal (dev + prod)

This guide explains how to keep Docker builds fast and images small while ensuring the image contains the Python and Rust dependencies you need (no venv required). It covers three goals:

- Small build context
- Minimal runtime image (production)
- Practical development image (dev toolchain + dev deps)

---

**1. Keep the build context tiny**

- Add a `.dockerignore` at the repository root that excludes large directories and build artifacts. Example (already added):

  - `.git/`, `target/`, `backend/target/`, `htmlcov/`, `data/`, `.venv/`, `venv/`, `node_modules/`, `.pytest_cache/`, `.mypy_cache/`, `.vscode/`

- Verify the context size before building:

  ```bash
  # prints bytes that would be sent as context (approx)
  tar -c --exclude-from=.dockerignore -f - . | wc -c
  ```

**Why:** Docker sends the entire build context to the daemon. Excluding big folders speeds transfer and reduces daemon memory/IO.

---

**2. Use multi-stage builds (already in `docker/Dockerfile`)**

- Pattern:
  - `rust-wheel-builder` (build Rust extension; produce wheels)
  - `dev` (developer image: rust toolchain + dev packages)
  - `runtime` (production: only runtime packages + prebuilt wheel installed)

- Use `--target dev` when you want an interactive dev image, and `--target runtime` for production.

**Build commands**

```bash
# Dev image (includes Rust toolchain and dev deps):
docker build -f docker/Dockerfile --target dev -t tsi:dev .

# Production image (runtime only):
docker build -f docker/Dockerfile --target runtime -t tsi:prod .
```

Or via Compose:

```bash
docker compose -f docker/docker-compose.yml -f .devcontainer/docker-compose.devcontainer.yml build app
```

---

**3. Install only required packages**

- Apt packages: use `--no-install-recommends` and install only the system libs you need for building/for runtime. Always clean apt lists in the same RUN to keep layers small:

```dockerfile
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential libpq-dev patchelf pkg-config curl \
 && rm -rf /var/lib/apt/lists/*
```

- Python packages: install globally (no venv) using `pip install --no-cache-dir` so wheels are installed and pip cache is not stored in image layers.

```dockerfile
RUN pip install --no-cache-dir -r requirements.base.txt
```

**Note:** Keep `requirements.base.txt` minimal — separate dev-only requirements into a `requirements.dev.txt` installed only in the `dev` target.

---

**4. Build Rust extension efficiently**

- Use the `rust-wheel-builder` stage to produce wheels via `maturin build` and copy the resulting wheel(s) into the runtime stage. This keeps the Rust toolchain out of the final image.

- If you must include Rust in the image (dev image), keep it limited to the `dev` target only.

- Use BuildKit caching for cargo to speed rebuilds (enable BuildKit and add cache mounts):

```dockerfile
# Example inside a build stage that uses BuildKit cache
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    maturin build --release --manifest-path backend/Cargo.toml -o /tmp/wheels
```

Enable BuildKit when building:

```bash
DOCKER_BUILDKIT=1 docker build -f docker/Dockerfile --target dev -t tsi:dev .
```

---

**5. Remove dev-only artifacts between stages**

- Keep the runtime stage minimal: install runtime-only Python packages, copy only the app sources and prebuilt wheels, and do not copy the entire repo.

Example runtime stage (already similar in `docker/Dockerfile`):

```dockerfile
FROM python:3.11-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends libpq5 libgomp1 && rm -rf /var/lib/apt/lists/*
COPY requirements.base.txt ./
RUN pip install --no-cache-dir -r requirements.base.txt
COPY --from=rust-wheel-builder /tmp/wheels /tmp/wheels
RUN pip install --no-cache-dir /tmp/wheels/*.whl && rm -rf /tmp/wheels
COPY src/ ./src/
```

---

**6. Recommendation: split `requirements` and dev deps**

- `requirements.base.txt` — runtime dependencies only.
- `requirements.dev.txt` — dev/test/lint tools (ruff, black, pytest, mypy, pandas-stubs, types-*) installed only in `dev` target.

This prevents installing heavy dev deps in runtime images.

---

**7. Optional CI optimization (fast reproducible builds)**

- In CI, build the Rust wheel once (in a dedicated builder job) and publish it to an artifact registry (S3, GitHub Packages, private PyPI). The production Dockerfile can `pip install` that wheel directly, eliminating Rust toolchain entirely from CI and local `docker build`.

CI snippet (conceptual):

```yaml
# build-wheel job
jobs:
  build-wheel:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build wheel
        run: |
          DOCKER_BUILDKIT=1 docker build -f docker/Dockerfile --target rust-wheel-builder -t wheel-builder .
          docker run --rm wheel-builder cat /tmp/wheels/*.whl > tsi_extension.whl
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: tsi_wheels
          path: tsi_extension.whl
```

---

**8. Checklist to follow now**

- [ ] Keep `.dockerignore` up to date (exclude new large folders).
- [ ] Split `requirements.base.txt` vs `requirements.dev.txt`.
- [ ] Use BuildKit caching for Cargo and pip where helpful.
- [ ] Build `runtime` image for production and `dev` image for day-to-day development.

---

If you want, I can:

- Add a small `requirements.dev.txt` and update the Dockerfile to install it only in the `dev` target.
- Add BuildKit cache lines to the Dockerfile and test a dev image build to measure improved build time.
- Add a CI job to prebuild the wheel and demonstrate building `tsi:prod` without Rust toolchain.

Tell me which of these you want next and I will implement it.
