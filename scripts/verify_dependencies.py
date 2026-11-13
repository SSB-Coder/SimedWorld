"""Exercise the installed project environment without running a world simulation."""

import argparse
from datetime import datetime, timezone
import importlib.metadata
import json
from pathlib import Path
import re
import subprocess
import sys
import traceback


ROOT = Path(__file__).resolve().parents[1]


def verify() -> dict:
    if Path(sys.prefix).resolve() != (ROOT / ".venv").resolve():
        raise RuntimeError("Run with .venv/Scripts/python.exe so global packages are not tested by mistake")
    import matplotlib
    matplotlib.use("Agg")
    import numpy as np
    import pandas as pd
    import torch
    from ncps.torch import CfC, LTC

    torch.manual_seed(42)
    array_result = torch.from_numpy(np.array([1.0, 2.0], dtype=np.float32)).sum().item()
    if array_result != 3.0:
        raise RuntimeError("NumPy/PyTorch interchange failed")
    devices = ["cpu"] + (["cuda"] if torch.cuda.is_available() else [])
    controllers = []
    for device in devices:
        for model_type in (CfC, LTC):
            model = model_type(input_size=3, units=8).to(device)
            inputs = torch.randn(2, 4, 3, device=device)
            outputs, state = model(inputs)
            if tuple(outputs.shape) != (2, 4, 8) or not torch.isfinite(outputs).all().item():
                raise RuntimeError(f"Invalid {model_type.__name__} output on {device}")
            outputs.square().mean().backward()
            gradients = [p.grad for p in model.parameters() if p.grad is not None]
            if not gradients or not all(torch.isfinite(g).all().item() for g in gradients):
                raise RuntimeError(f"Invalid {model_type.__name__} gradient on {device}")
            controllers.append({"model": model_type.__name__, "device": device,
                                "output_shape": list(outputs.shape), "forward_backward": "passed"})
    if "cuda" in devices:
        value = (torch.ones((2, 2), device="cuda") @ torch.ones((2, 2), device="cuda")).sum().item()
        torch.cuda.synchronize()
        if value != 8.0:
            raise RuntimeError("GPU arithmetic failed")
    tools = {}
    for tool in ("maturin", "cmake"):
        result = subprocess.run([str(ROOT / ".venv/Scripts" / f"{tool}.exe"), "--version"],
                                check=True, capture_output=True, text=True)
        tools[tool] = result.stdout.strip()
    installed = sorted([{"name": dist.metadata["Name"], "version": dist.version}
                        for dist in importlib.metadata.distributions()], key=lambda item: item["name"].lower())
    from audit_dependencies import read_packages
    def normalize(name):
        return re.sub(r"[-_.]+", "-", name).lower()
    expected = {normalize(item["name"]): item["installed_version"]
                for lock in (ROOT / "dependencies/python-win-py313.lock", ROOT / "dependencies/build-tools-win-py313.lock")
                for item in read_packages(lock, "python")}
    actual = {normalize(item["name"]): item["version"] for item in installed}
    # The locally built application is separate from third-party dependency locks.
    local_project_version = actual.pop("world-microcosm", None)
    if expected != actual:
        raise RuntimeError("Installed packages do not exactly match the reviewed runtime/build-tool locks")
    artifact_dir = ROOT / "artifacts/setup"
    artifact_dir.mkdir(parents=True, exist_ok=True)
    (artifact_dir / "installed-python.txt").write_text(
        "".join(f"{item['name']}=={item['version']}\n" for item in installed
                if normalize(item["name"]) != "world-microcosm"), encoding="utf-8")
    return {"status": "passed", "python": sys.version, "executable": sys.executable,
            "torch": torch.__version__, "cuda_build": torch.version.cuda,
            "cuda_available": torch.cuda.is_available(),
            "gpu": torch.cuda.get_device_name(0) if "cuda" in devices else None,
            "numpy": np.__version__, "pandas": pd.__version__, "matplotlib": matplotlib.__version__,
            "numpy_torch_interchange": "passed", "controllers": controllers,
            "installed_versions_match_locks": True,
            "local_project_version": local_project_version,
            "tools": tools, "installed_packages": installed,
            "scope": "Small dependency smoke checks; not evolutionary, ecological, or performance results."}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/setup/python-verification.json")
    args = parser.parse_args()
    try:
        report = verify()
        code = 0
    except Exception as error:
        report = {"status": "failed", "error": str(error), "traceback": traceback.format_exc()}
        code = 1
    report["checked_at_utc"] = datetime.now(timezone.utc).isoformat()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: report[key] for key in ("status", "torch", "cuda_available", "gpu", "controllers", "tools", "error") if key in report}, indent=2))
    return code


if __name__ == "__main__":
    sys.exit(main())
