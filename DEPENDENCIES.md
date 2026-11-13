# Installed dependencies and verified builds

The workspace builds the mixed Python/Rust `world-microcosm` package and a native Bevy observer. The core supplies diagnostic and evolving biological modes, stable body/neural mutation, and full native continuation. Python runs the inherited sparse neural graphs and coordinated saves; Bevy observes that owner. Fixed wired CfC/LTC candidates have interface/inheritance checks. Lifetime plasticity, social signaling, and Rapier mechanics remain deferred.

## Project environment

Use `.venv\Scripts\python.exe` from this project. The existing global Python environment was not upgraded. It still contains torch 2.11.0+cu128, for which the advisory check returned [GHSA-rrmf-rvhw-rf47](https://osv.dev/vulnerability/GHSA-rrmf-rvhw-rf47), concerning torch.jit.script memory corruption. The project instead uses the fixed-version range below. This is not an audit of unrelated global packages/projects.

| Direct package | Installed project version | Purpose |
| --- | --- | --- |
| torch | 2.14.1+cu126 | CPU/GPU neural computation; CUDA 12.6 build works with the existing driver |
| numpy | 2.5.4 | Array computation and tensor interchange |
| ncps | 1.0.1 | LTC/CfC controller experiments |
| maturin | 1.16.0 | Build/package Rust Python extensions |
| cmake | 4.4.4 | Project-local native build utility |
| matplotlib | 3.11.2 | Analysis plots |
| pandas | 3.1.1 | Recorded-result analysis |
| pytest | 9.2.0 | Python test tooling |
| pip-audit | 2.10.1 | Known-vulnerability checks |
| pip | 26.3 | Package inspection/consistency tooling inside the virtual environment |

The environment uses Python 3.13.7. The 55 third-party packages, including transitive dependencies and CMake, match the reviewed locks exactly. The locally built editable distribution, `world-microcosm` 0.1.0, brings the total to 56 installed distributions. The verifier separates that application from the third-party locks and advisory inventory. Third-party dependencies were installed as hash-checked published wheels from PyPI and the official PyTorch CUDA index.

The distribution is named `world-microcosm`; its Python import is `world`, exposing `Habitat`, `ResourceLedger`, `FixtureWorld`, and the original scalar diagnostics. `world.runtime.BridgeRuntime` coordinates the fixture and controller registry. Package metadata supports Python 3.13 and requires NumPy 2.5.4 and PyTorch 2.14.1. The PyTorch requirement accepts the installed `+cu126` build; use the reviewed research locks to reproduce that CUDA selection. Only Windows x64 / CPython 3.13 has been verified.

Pinned input and lock files:

- [dependencies/python.in](dependencies/python.in)
- [dependencies/python-win-py313.lock](dependencies/python-win-py313.lock)
- [dependencies/build-tools.in](dependencies/build-tools.in)
- [dependencies/build-tools-win-py313.lock](dependencies/build-tools-win-py313.lock)

The runtime/tool wheel set in the first lock has published sizes totaling approximately 2.49 GiB; CMake adds a roughly 40 MiB wheel. Actual cache/network/disk use varies. The current system NVIDIA driver was not changed, and a separate CUDA Toolkit was not installed.

## Rust dependencies

[rust-toolchain.toml](rust-toolchain.toml) pins Rust 1.97.1 for this project, with rustfmt and clippy. The previous global stable default remains unchanged. The official rustup operation also self-updated the rustup manager from 1.29.0 to 1.29.3.

The production [Cargo.toml](Cargo.toml) defines `world_core`, `world_genetics`, `world_bindings`, and `world_viewer`. [Cargo.lock](Cargo.lock) contains their active dependency resolution. The core is Python/rendering-independent; genetics provides immutable birth genomes, stable body/neural identities, clone inheritance, and validated mutation operators; bindings use PyO3 and call the core. The temporary `dependencies/rust/` source package is retired and archived under `artifacts/setup/retired-dependency-scaffold/`. Its compiled cache was moved into root `target/`.

The active members use PyO3, Rust NumPy, Serde, serde_json, and Bevy. serde_json's float_roundtrip feature preserves f64 checkpoint values. Bevy's BMP feature permits diagnostic captures without another image-format dependency. Other reviewed direct versions remain workspace declarations and do not enter the active lock until used. Lock changes are reviewed and audited before building.

| Direct crate | Pinned version | Purpose |
| --- | --- | --- |
| bevy | 0.20.1 | Windows-focused 2D rendering/app dependency |
| rapier2d | 0.36.1 | Physics, serialization, and deterministic-mode support |
| pyo3 | 0.30.1 | Python native-extension interface |
| numpy | 0.30.0 | Rust/NumPy integration; distinct from the Python numpy package |
| serde | 1.0.229 | Structured serialization |
| serde_json | 1.0.154 | JSON configuration and records |
| rand | 0.10.3 | Randomness |
| rand_chacha | 0.10.0 | Explicit seeded RNG |
| rand_distr | 0.6.0 | Mutation distributions |
| rayon | 1.12.0 | Optional data parallelism |

The active root lock contains 470 registry packages, all with versions and checksums from the previously reviewed graph. Its current OSV check has no known advisory matches. Cargo metadata required fetching portable-atomic-util 0.2.8 at its already locked version. The original 497-package lock and all-feature dependency build are historical setup evidence; their reports and source inputs remain archived.

The active Bevy declaration uses Windows-focused features, excluding unnecessary Linux/Wayland dependencies. Additional locked target-platform source archives were fetched from Cargo's configured registry for metadata resolution; no dependency versions were upgraded. The current bindings have an `extension-module` feature used by maturin. The old scaffold's `viewer`, `bridge`, and `parallel` groups are retired; `world_viewer` uses Bevy's built-in text rather than egui.

## Deliberately deferred

`bevy_egui` 0.43.1 was not included in the final dependency graph. Its resolved font path includes ttf-parser 0.25.1, which has [RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html), an unmaintained-package advisory. This is informational maintenance status, not evidence of a current exploit. Under the requested conservative policy, UI integration waits for a maintained dependency path or an explicit later review. See [dependencies/deferred.json](dependencies/deferred.json).

`cargo-audit` has not been installed. Rust packages were checked directly against OSV, which includes exported RustSec advisories; no claim is made that the cargo-audit command itself ran. The SDK blocker is resolved; a dedicated Rust audit tool remains optional future tooling.

## Windows build prerequisites: resolved

Windows SDK **10.0.26100.0** is now present under `C:\Program Files (x86)\Windows Kits\10`, including x64 `kernel32.lib`, `ucrt.lib`, and Windows/UCRT headers. Together with MSVC **14.44.35207**, it compiled, linked, and ran the existing tiny Rust platform probe. The previous missing-`kernel32.lib` blocker is resolved; no further prerequisite installation is currently identified.

For a future clean machine, use Visual Studio Installer to add the Windows SDK and MSVC v143 x64/x86 build tools, then open a new terminal. The full Visual Studio IDE is not required. The completed dependency build only set compiler/SDK paths inside its process; it did not change machine-wide environment settings.

The existing installer executable has a valid Microsoft Corporation Authenticode signature. Use that installed Microsoft tool rather than third-party SDK downloads. The SDK requirement is documented in the [Rust Windows prerequisites](https://rust-lang.github.io/rustup/installation/windows-msvc.html) and [Bevy setup guide](https://bevy.org/learn/quick-start/getting-started/setup/).

CMake is already installed in `.venv`; the NVIDIA driver, Python installation, and full Visual Studio IDE do not need replacement for the verified Python setup.

## Verification commands

Python and GPU checks, without requiring activation:

```powershell
.\.venv\Scripts\python.exe scripts/verify_dependencies.py
.\.venv\Scripts\python.exe -m pip check
.\.venv\Scripts\python.exe -m unittest discover -s tests -v
```

Build/install the editable package, make a native call, or build a wheel:

```powershell
.\scripts\build_native.ps1
.\.venv\Scripts\python.exe -c "import world; print(world.core_version()); print(world.checked_add(2, 3))"
.\scripts\build_native.ps1 -Wheel
```

The expected diagnostic output is `0.1.0` and `5`. Wheel output is under `artifacts/wheels/`. Source/build configuration follows the official [maturin mixed-project layout](https://www.maturin.rs/project_layout.html) and [PyO3 module interface](https://pyo3.rs/v0.30.1/module.html).

The build helper uses the project environment, checks the root lock against OSV, then runs locked/offline maturin with two Cargo jobs. The advisory request requires network access; Cargo builds and package installation use cached/local inputs. Compiler paths and package-manager environment variables are process-local and restored afterward. No execution-policy or machine-wide environment changes are made.

The verification helper checks Python, rechecks Rust advisories, prepares the same MSVC/SDK paths, and runs `cargo check` on the production workspace:

```powershell
.\scripts\verify_dependencies.ps1 -IncludeRust
```

If local PowerShell script execution is restricted, keep the policy unchanged and use the **x64 Native Tools Command Prompt for VS 2022**, change to this project, and run:

```text
cargo check --locked --offline --workspace --all-features --jobs 2
```

For an editable native build without PowerShell helpers, use that Native Tools Command Prompt:

```bat
cd /d C:\Thought\Projects\World
set "PYO3_PYTHON=C:\Thought\Projects\World\.venv\Scripts\python.exe"
set "VIRTUAL_ENV=C:\Thought\Projects\World\.venv"
set "PIP_NO_INDEX=1"
set "UV_OFFLINE=true"
.venv\Scripts\maturin.exe develop --locked --offline --jobs 2
```

Both editable installation and wheel generation passed. Compiled files are under root `target/`; editable installation places a generated `.pyd` beside `python/world/__init__.py`. Generated binaries are ignored by Git. `cargo check` omits normal final code generation and is not a substitute for building/installing and importing the extension.

The helper does not install SDKs, change machine-wide PATH, disable security settings, or publish anything. It restores its temporary environment changes when finished.

## Audit and verification evidence

- [Installed Python advisory check](artifacts/setup/python-installed-audit.json): 55 packages, no known advisory matches.
- [pip-audit report](artifacts/setup/pip-audit.json): independent installed-environment OSV audit.
- [Selected Rust advisory check](artifacts/setup/rust-selected-audit.json): 497 packages, no known advisory matches.
- [Active workspace advisory check](artifacts/setup/rust-workspace-audit.json): 470 packages, no known advisory matches.
- [Python/CPU/GPU smoke results](artifacts/setup/python-verification.json): exact installed versions, NumPy/PyTorch interchange, CfC/LTC forward/backward tests on CPU and CUDA, maturin/CMake version checks.
- [Published runtime wheel manifest](artifacts/setup/python-artifacts.json): selected artifacts, hashes, and published sizes.
- [Native prerequisite report](artifacts/setup/native-build.json): SDK presence and successful native compile/link/run probe.
- [Rust build result](artifacts/setup/rust-build.json) and [build log](artifacts/setup/rust-build.log): successful locked, offline, all-feature dependency-scaffold compilation.
- [Native package verification](artifacts/setup/native-package-verification.json): historical scalar-package build and integration results.
- [Bridge verification](artifacts/setup/bridge-verification.json): historical bridge package, contract, backend, continuation, and wheel results.
- [Resource accounting verification](artifacts/setup/accounting-verification.json): historical native lifecycle, reference agreement, package regression, and wheel results. The accounting implementation adds no registry or third-party Python versions.
- [Habitat verification](artifacts/setup/habitat-verification.json): historical scheduled/local producer behavior, conservation, repeatability, rollback, package regression, and isolated-wheel results. No registry or Python dependency versions were added.

- [Ecosystem and viewer verification](artifacts/setup/ecosystem-verification.json): historical ecological-cycle, continuation, observer, regression, package, and source-hash evidence.

- [Evolving bodies/brains verification](artifacts/setup/evolution-verification.json): current inherited control, mutation/topology, anatomy/damage, experiments, observer protocol, coordinated continuation, package, and source evidence. No registry or third-party Python versions were added.

- [Viewer/runtime correction verification](artifacts/setup/viewer-fix-verification.json): real launcher, clock pacing, measured speed, native compilation optimizations, plain output, current regression/package results and source hashes. The dependency versions remain unchanged.

The Python suite covers the resource reference, native/reference lifecycle agreement, conservation, funded inheritance, invalid-operation rollback, native installation, strict batch routing/rejection, proposed-memory rollback, CPU/CUDA agreement, empty cohorts, and CPU continuation. Rust tests independently exercise accounting, staged-audit rollback, configuration, identity, and clock bounds. The current report records executed counts and backend results. Wheel verification uses a separate environment with the locked runtime dependencies. These checks do not establish population capacity, useful learning, ecological realism, or absence of unknown vulnerabilities. Advisory checks are time-stamped snapshots, not security guarantees or complete source-code reviews.

## Reproducing the installation later

Review any new advisory results before executing new package versions. The Windows/Python locks are platform-specific; do not assume they describe Linux/macOS or another Python ABI.

```powershell
uv --no-config venv .venv --python 3.13 --no-python-downloads
uv --no-config pip sync dependencies/python-win-py313.lock dependencies/build-tools-win-py313.lock --python .venv\Scripts\python.exe --torch-backend cu126 --default-index https://pypi.org/simple --only-binary :all: --require-hashes --strict --link-mode copy
cargo fetch --locked
.\scripts\build_native.ps1
```

Do not recreate an existing environment merely to run verification. `uv pip sync` restores only third-party locked packages and removes the local editable application; rerun `build_native.ps1` afterward. Package downloads live in normal user-level caches, compiled Rust output in root `target/`, project packages in `.venv`, and project-specific Rust selection in rust-toolchain.toml.

Current circle repair uses only the existing Rust and reviewed Python dependencies. `scripts/plain_command.py` uses the standard library to decode native UTF-8 before PowerShell sees it and preserve command exit codes. Windows run ownership uses the standard-library OS file lock. [Circle repair verification](artifacts/setup/circle-repair-verification.json) supersedes earlier functional reports for the default launcher; their historical results remain available.


The ecological extensions and release tooling add no third-party dependencies. Development/test Cargo profiles now disable debug symbols and incremental caching; all existing package optimization levels remain. The final native wheel passed the complete Python suite in a separate environment before cleanup of that duplicate environment was requested. Automatic approval review blocked its recursive deletion even after user confirmation, so it remains available for manual removal. This storage change does not remove the main CUDA-enabled runtime or saved neural parameters. See the expanded verification report for the actual test count, package hashes and cleanup outcome.
