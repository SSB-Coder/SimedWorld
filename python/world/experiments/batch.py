"""Bounded independent runs with checkpoints, failures and declared controls."""

from concurrent.futures import ProcessPoolExecutor
import json
import hashlib
from pathlib import Path
import time
import traceback

from ..archive import SubjectArchive, digest
from ..ecosystem import EcosystemRuntime, write_json
from ..taxonomy import classify
from .ecosystem import ecosystem_preset, hunting_scenario, social_scenario, ablation


def run_trial(request):
    import torch

    torch.set_num_threads(request.get("threads", 1))
    directory = Path(request["output"])
    directory.mkdir(parents=True, exist_ok=True)
    runtime = None
    started = time.perf_counter()
    try:
        if request.get("resume"):
            runtime = EcosystemRuntime.load(
                request["resume"], request.get("device", "cpu")
            )
        else:
            if request.get("scenario", "ecosystem") == "hunting":
                rules, config = hunting_scenario(
                    request["rules"], request["seed"], request.get("hunters", 4)
                )
            elif request.get("scenario") in {"herding", "defense"}:
                rules, config = social_scenario(
                    request["rules"], request["seed"], request["scenario"] == "defense"
                )
            else:
                rules, config = ecosystem_preset(
                    request["rules"], request["seed"], request.get("scale", 1)
                )
            config = ablation(config, request.get("control", "full"))
            runtime = EcosystemRuntime.create(
                rules,
                config,
                request.get("device", "cpu"),
                request.get("controller", "sparse"),
            )
        runtime.archive = SubjectArchive(
            directory / "subjects", request.get("archive_limit", 128)
        )
        from .. import _native

        sources = [Path(_native.__file__), *Path(__file__).parents[1].rglob("*.py")]
        initial = runtime.snapshot()
        write_json(
            directory / "manifest.json",
            {
                "schema_version": 1,
                "rules": initial["world"]["rules"],
                "configuration": initial["world"]["config"],
                "device": str(runtime.brains.device),
                "controller": runtime.brains.kind,
                "start_tick": runtime.tick,
                "source_sha256": {
                    p.relative_to(Path(__file__).parents[1]).as_posix(): hashlib.sha256(
                        p.read_bytes()
                    ).hexdigest()
                    for p in sources
                },
            },
        )
        samples = []
        last = runtime.tick
        event_cursor = runtime.next_event
        interval = request.get("sample_decisions", 100)
        checkpoints = request.get("checkpoint_decisions", 1000)
        for decision in range(request["decisions"]):
            runtime.advance()
            receipt = runtime.events_since(event_cursor)
            if receipt["events"]:
                with (directory / "events.jsonl").open("a", encoding="utf-8") as stream:
                    for event in receipt["events"]:
                        stream.write(json.dumps(event, allow_nan=False) + "\n")
            event_cursor = receipt["next_event"]
            if decision % interval == 0 or decision + 1 == request["decisions"]:
                row = runtime.telemetry()
                row.update(
                    wall_seconds=time.perf_counter() - started,
                    neurons=len(runtime.brains.keys),
                    connections=len(runtime.brains.weights),
                )
                samples.append(row)
                with (directory / "telemetry.jsonl").open(
                    "a", encoding="utf-8"
                ) as stream:
                    stream.write(json.dumps(row, allow_nan=False) + "\n")
            if (decision + 1) % checkpoints == 0:
                runtime.save(directory / "checkpoint.json")
        runtime.save(directory / "checkpoint.json")
        state = runtime.snapshot()
        living = [a["id"] for a in runtime._frame["agents"]]
        if living:
            counts = {o["id"]: o["age"] for o in state["world"]["ledger"]["organisms"]}
            runtime.archive.capture(
                runtime, max(living, key=lambda identity: counts[identity]), "survivor"
            )
            children = {identity: 0 for identity in living}
            for organism in state["world"]["ledger"]["organisms"]:
                if organism["parent_id"] in children:
                    children[organism["parent_id"]] += 1
            sizes = {
                a["id"]: len(a["brain"]["nodes"]) + len(a["brain"]["connections"])
                for a in runtime._frame["agents"]
            }
            runtime.archive.capture(
                runtime,
                max(
                    living, key=lambda identity: (children[identity], counts[identity])
                ),
                "ecological",
            )
            runtime.archive.capture(
                runtime,
                max(
                    living,
                    key=lambda identity: (
                        children[identity] / max(1, sizes[identity]),
                        -sizes[identity],
                    ),
                ),
                "compact",
            )
        result = {
            "schema_version": 1,
            "kind": "world.study",
            "status": "completed",
            "request_sha256": digest(request),
            "request": request,
            "start_tick": last,
            "end_tick": runtime.tick,
            "wall_seconds": time.perf_counter() - started,
            "actual_speed": (runtime.tick - last)
            * state["world"]["rules"]["physics_dt"]
            / (time.perf_counter() - started),
            "samples": samples,
            "audit": runtime.audit(),
            "taxonomy": classify(state["world"]["ledger"]["organisms"]),
            "limits": runtime.telemetry()["limits"],
            "subjects_retained": len(runtime.archive.paths),
        }
        if request.get("scenario") == "hunting":
            prey = state["world"]["ledger"]["organisms"][0]
            web = runtime.telemetry()["food_web"]
            hunters = request.get("hunters", 4)
            result["hunting"] = {
                "hunters": hunters,
                "prey_alive": prey["alive"],
                "capture": not prey["alive"]
                and prey["death_cause"] in {"consumption", "damage"},
                "food_per_hunter": sum(
                    web["intake"].get(str(i), 0.0) for i in range(2, hunters + 2)
                )
                / hunters,
                "communication_energy": runtime.telemetry()["costs"].get(
                    "communication", 0.0
                ),
            }
        write_json(directory / "result.json", result)
        return {"status": "completed", "result": str(directory / "result.json")}
    except Exception as error:
        if runtime is not None:
            runtime.save(directory / "failure-checkpoint.json")
        failure = {
            "status": "failed",
            "error": str(error),
            "traceback": traceback.format_exc(),
            "request": request,
        }
        write_json(directory / "failure.json", failure)
        return failure


def run_batch(requests, workers=1):
    if type(workers) is not int or not 1 <= workers <= 8:
        raise ValueError("workers must lie within [1,8]")
    paths = [str(Path(r["output"]).resolve()).casefold() for r in requests]
    if len(set(paths)) != len(paths):
        raise ValueError("independent trials require separate output directories")
    for r in requests:
        for key in ["decisions", "sample_decisions", "checkpoint_decisions", "threads"]:
            value = r.get(
                key,
                100
                if key == "sample_decisions"
                else 1000
                if key == "checkpoint_decisions"
                else 1,
            )
            if type(value) is not int or value < 1:
                raise ValueError(f"invalid {key}")
        if (
            Path(r["output"], "result.json").exists()
            or Path(r["output"], "telemetry.jsonl").exists()
        ):
            raise ValueError("study output already exists; choose a new directory")
    if workers == 1:
        return [run_trial(r) for r in requests]
    with ProcessPoolExecutor(max_workers=workers) as pool:
        return list(pool.map(run_trial, requests))
