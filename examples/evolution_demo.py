import argparse
import json
from pathlib import Path
import subprocess
import time
import torch

from world.control import RunControl, RunLease
from world.ecosystem import EcosystemRuntime, write_json
from world.telemetry import RunRecorder
from world.archive import SubjectArchive


ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(
        description="Run inherited neural organisms and birth-time evolution."
    )
    parser.add_argument(
        "--rules", type=Path, default=ROOT / "configs/food_web_rules.json"
    )
    parser.add_argument(
        "--config", type=Path, default=ROOT / "configs/food_web_habitat.json"
    )
    parser.add_argument("--resume", type=Path)
    parser.add_argument("--device", choices=["cpu", "cuda"], default="cpu")
    parser.add_argument("--controller", choices=["sparse", "ltc"], default="sparse")
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument("--decisions", type=int, default=300)
    parser.add_argument("--sample-every", type=int, default=20)
    parser.add_argument(
        "--checkpoint", type=Path, default=ROOT / "artifacts/evolution-checkpoint.json"
    )
    parser.add_argument(
        "--output", type=Path, default=ROOT / "artifacts/evolution-run.json"
    )
    parser.add_argument(
        "--events", type=Path, default=ROOT / "artifacts/evolution-events.jsonl"
    )
    parser.add_argument(
        "--stream", type=Path, default=ROOT / "artifacts/evolution-observer.json"
    )
    parser.add_argument(
        "--commands", type=Path, default=ROOT / "artifacts/evolution-commands"
    )
    parser.add_argument("--viewer", action="store_true")
    parser.add_argument("--viewer-executable", type=Path)
    parser.add_argument("--viewer-run-for-ticks", type=int)
    parser.add_argument("--viewer-capture", type=Path)
    parser.add_argument("--viewer-select", type=int)
    parser.add_argument("--interactive", action="store_true")
    parser.add_argument("--paused", action="store_true")
    args = parser.parse_args()
    if (
        args.threads < 1
        or args.decisions < 0
        or args.sample_every < 1
        or args.paused
        and not args.interactive
        or args.viewer_run_for_ticks is not None and args.viewer_run_for_ticks < 0
        or args.viewer_capture is not None and args.viewer_run_for_ticks is None
    ):
        parser.error(
            "invalid run length/sample interval; paused requires interactive mode"
        )
    torch.set_num_threads(args.threads)
    with RunLease(args.stream):
        run(args)


def run(args):
    executable = (
        getattr(args, "viewer_executable", None)
        or ROOT / "target/debug/world_viewer.exe"
    )
    if args.viewer and not executable.exists():
        raise FileNotFoundError(
            "viewer executable is missing; run scripts/run_evolution.ps1"
        )
    if args.resume:
        runtime = EcosystemRuntime.load(args.resume, args.device)
    else:
        runtime = EcosystemRuntime.create(
            json.loads(args.rules.read_text()),
            json.loads(args.config.read_text()),
            args.device,
            getattr(args, "controller", "sparse"),
        )
    control = RunControl(
        runtime,
        args.checkpoint,
        args.stream,
        args.commands,
        args.paused,
    )
    runtime.archive = SubjectArchive(args.output.parent / "subjects")
    control.publish()
    viewer = None
    viewer_log = None
    samples = [runtime.summary(compact=True)]
    recorder = RunRecorder(runtime, args.output)
    args.events.parent.mkdir(parents=True, exist_ok=True)
    last_event = runtime.next_event
    decisions = 0
    segment = 0
    started = time.perf_counter()
    last_publication = started
    failure = None
    try:
        if args.viewer:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            viewer_log = args.output.with_suffix(".viewer.log").open("w", encoding="utf-8")
            viewer_arguments = [
                str(executable), "--observe", str(args.stream),
                "--commands", str(args.commands), "--checkpoint", str(args.checkpoint),
            ]
            for attribute, flag in [
                ("viewer_run_for_ticks", "--run-for-ticks"),
                ("viewer_capture", "--capture"),
                ("viewer_select", "--select"),
            ]:
                value = getattr(args, attribute, None)
                if value is not None:
                    viewer_arguments.extend([flag, str(value)])
            viewer = subprocess.Popen(
                viewer_arguments,
                cwd=ROOT,
                stdout=viewer_log,
                stderr=subprocess.STDOUT,
                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
            )
        with args.events.open(
            "a" if args.resume else "w", encoding="utf-8"
        ) as event_stream:
            while args.interactive or decisions < args.decisions:
                if viewer is not None and viewer.poll() is not None:
                    if viewer.returncode != 0:
                        raise RuntimeError(
                            f"Viewer exited with code {viewer.returncode}; "
                            f"see {args.output.with_suffix('.viewer.log')}"
                        )
                    break
                frame_started = time.perf_counter()
                previous_status = (
                    control.paused,
                    control.speed,
                    control.last_command,
                    control.status,
                )
                previous_runtime = control.runtime
                advanced = control.update(
                    frame_started if args.interactive or args.viewer else None
                )
                runtime = control.runtime
                if runtime is not previous_runtime:
                    runtime.archive = SubjectArchive(args.output.parent / "subjects")
                    segment += 1
                    last_event = (
                        control.last_advance_start_event
                        if advanced
                        else runtime.next_event
                    )
                    event_stream.write(
                        json.dumps(
                            {
                                "session": control.session,
                                "segment": segment,
                                "kind": "restore",
                                "tick": runtime.tick,
                            }
                        )
                        + "\n"
                    )
                if advanced:
                    decisions += 1
                    recorder.sample(runtime)
                    receipt = runtime.events_since(last_event)
                    events = receipt["events"]
                    for event in events:
                        if event["sequence"] >= last_event:
                            event_stream.write(
                                json.dumps(
                                    {
                                        "session": control.session,
                                        "segment": segment,
                                        "event": event,
                                    },
                                    allow_nan=False,
                                )
                                + "\n"
                            )
                    last_event = receipt["next_event"]
                    if decisions % args.sample_every == 0:
                        row = runtime.summary(compact=True)
                        samples.append(
                            {k: v for k, v in row.items() if k != "diagnostics"}
                            | {
                                "diagnostics": {
                                    k: v
                                    for k, v in row["diagnostics"].items()
                                    if k != "events"
                                }
                            }
                        )
                        print(
                            f"tick={row['tick']} alive={row['living_count']} births={row['ecology']['births']} "
                            f"deaths={row['ecology']['deaths']} mutations={row['diagnostics']['accepted_mutations']}",
                            flush=True,
                        )
                if args.interactive or args.viewer:
                    now = time.perf_counter()
                    if now - last_publication >= 0.1 or previous_status != (
                        control.paused,
                        control.speed,
                        control.last_command,
                        control.status,
                    ):
                        control.publish()
                        last_publication = time.perf_counter()
                    if not advanced:
                        time.sleep(0.002)
                elif decisions % args.sample_every == 0:
                    control.publish()
    except KeyboardInterrupt:
        control.paused = True
    except Exception as error:
        failure = f"{type(error).__name__}: {error}"
        raise
    finally:
        runtime = control.runtime
        recorder.sample(runtime, force=True)
        control.paused = True
        control.status = (
            f"Run failed; state saved: {failure}"
            if failure else "Run finished; complete state saved"
        )
        runtime.save(args.checkpoint)
        control.publish()
        control.write_interventions(args.output.with_suffix(".interventions.json"))
        write_json(
            args.output,
            {
                "schema_version": 1,
                "samples": samples,
                "final": runtime.summary(),
                "audit": runtime.audit(),
                "error": failure,
                "decisions_executed": decisions,
                "wall_seconds": time.perf_counter() - started,
                "scope": "Independent circle organisms with bounded online plasticity and inherited learned parameters; legacy presets remain available.",
            },
        )
        if viewer is not None and viewer.poll() is None:
            viewer.terminate()
            viewer.wait(timeout=10)
        if viewer_log is not None:
            viewer_log.close()
    print(f"Saved complete ecosystem at tick {runtime.tick}; audit={runtime.audit()}")


if __name__ == "__main__":
    main()
