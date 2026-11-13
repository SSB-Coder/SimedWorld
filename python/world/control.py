from pathlib import Path
import os
import time
import uuid

from .ecosystem import EcosystemRuntime, read_json, write_json
from .brains.plastic import PlasticPopulation


class RunLease:
    def __init__(self, stream):
        self.path = Path(str(stream) + ".lock")
        self.file = None

    def __enter__(self):
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.file = self.path.open("a+b")
        self.file.seek(0, 2)
        if not self.file.tell():
            self.file.write(b"0")
            self.file.flush()
        self.file.seek(0)
        try:
            if os.name == "nt":
                import msvcrt

                msvcrt.locking(self.file.fileno(), msvcrt.LK_NBLCK, 1)
            else:
                import fcntl

                fcntl.flock(self.file.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError as error:
            self.file.close()
            raise RuntimeError(
                "This run directory already has an owner; choose another --stream or -RunDirectory."
            ) from error
        return self

    def __exit__(self, *error):
        try:
            self.file.seek(0)
            if os.name == "nt":
                import msvcrt

                msvcrt.locking(self.file.fileno(), msvcrt.LK_UNLCK, 1)
            else:
                import fcntl

                fcntl.flock(self.file.fileno(), fcntl.LOCK_UN)
        finally:
            self.file.close()


class RunControl:
    def __init__(
        self,
        runtime,
        checkpoint,
        stream=None,
        commands=None,
        paused=False,
        lightweight=None,
    ):
        self.runtime = runtime
        self.checkpoint = Path(checkpoint)
        self.stream = Path(stream) if stream else None
        self.commands = Path(commands) if commands else None
        self.paused = paused
        self.speed = 1
        self.single = 0
        self.session = uuid.uuid4().hex
        self.sequence = 0
        self.status = "Biological controller runtime"
        self.last_command = None
        self.interventions = []
        self.last_advance_start_event = None
        self._next_decision_wall = None
        self._pacing_speed = self.speed
        self._rate_wall = time.perf_counter()
        self._rate_time = runtime.elapsed_time
        self.actual_speed = 0.0
        self.lightweight = (
            isinstance(runtime.brains, PlasticPopulation)
            if lightweight is None
            else lightweight
        )
        self.selected = None
        if self.commands:
            self.commands.mkdir(parents=True, exist_ok=True)

    def publish(self):
        if self.stream:
            now = time.perf_counter()
            elapsed = now - self._rate_wall
            model_time = self.runtime.elapsed_time
            if self.paused or model_time < self._rate_time:
                self.actual_speed = 0.0
                self._rate_wall, self._rate_time = now, model_time
            elif elapsed >= 0.5:
                self.actual_speed = (model_time - self._rate_time) / elapsed
                self._rate_wall, self._rate_time = now, model_time
            self.sequence += 1
            write_json(
                self.stream,
                {
                    "schema_version": 1,
                    "kind": "world.display" if self.lightweight else "world.observer",
                    "session": self.session,
                    "sequence": self.sequence,
                    "paused": self.paused,
                    "speed": self.speed,
                    "actual_speed": self.actual_speed,
                    "status": self.status,
                    "last_command": self.last_command,
                    "state": self.runtime.display(self.selected)
                    if self.lightweight
                    else self.runtime.snapshot(),
                },
            )

    def apply(self, command):
        if not isinstance(command, dict) or set(command) != {
            "schema_version",
            "session",
            "id",
            "kind",
            "path",
        }:
            raise ValueError("unexpected observer command fields")
        if (
            type(command["schema_version"]) is not int
            or command["schema_version"] != 1
            or command["session"] != self.session
        ):
            raise ValueError("stale observer session")
        if (
            not isinstance(command["id"], str)
            or not command["id"]
            or not isinstance(command["path"], str)
        ):
            raise ValueError("invalid observer command identity/path")
        kind = command["kind"]
        previous = next(
            (e for e in self.interventions if e["id"] == command["id"]), None
        )
        if previous:
            if previous["kind"] != kind:
                raise ValueError(
                    "observer command identity was reused with a different action"
                )
            self.last_command = command["id"]
            return
        if kind == "pause":
            self.paused = not self.paused
        elif kind == "advance":
            self.paused = True
            self.single += 1
        elif kind == "faster":
            self.speed = min(16, self.speed * 2)
        elif kind == "slower":
            self.speed = max(1, self.speed // 2)
        elif kind == "save":
            self.runtime.save(command["path"] or self.checkpoint)
            self.status = f"Saved tick {self.runtime.tick}"
        elif kind == "load":
            candidate = EcosystemRuntime.load(
                command["path"] or self.checkpoint, str(self.runtime.brains.device)
            )
            selected = self.selected
            if selected is not None and not any(
                organism["id"] == selected
                for organism in candidate.display(selected)["organisms"]
            ):
                selected = None
            self.runtime = candidate
            self.lightweight = isinstance(candidate.brains, PlasticPopulation)
            self.selected = selected
            self.paused = True
            self.single = 0
            self._next_decision_wall = None
            self._rate_wall, self._rate_time = (
                time.perf_counter(),
                candidate.elapsed_time,
            )
            self.actual_speed = 0.0
            self.status = f"Loaded tick {self.runtime.tick} (paused)"
        elif kind == "inspect":
            identity = int(command["path"])
            if not 0 <= identity <= 2**64 - 1:
                raise ValueError("invalid inspection identity")
            self.selected = identity or None
        elif kind == "export":
            from .archive import SubjectArchive

            if self.selected is None:
                raise ValueError("select a neural organism before exporting")
            archive = SubjectArchive(self.checkpoint.parent / "subjects")
            path = archive.capture(self.runtime, self.selected, "inspection")
            self.status = f"Exported subject #{self.selected}: {path.name}"
        else:
            raise ValueError("unknown observer command")
        self.last_command = command["id"]
        self.interventions.append(
            {"tick": self.runtime.tick, "id": command["id"], "kind": kind}
        )
        self.interventions = self.interventions[-512:]

    def receive(self):
        if not self.commands:
            return
        for path in sorted(self.commands.glob("cmd-*.json")):
            try:
                command = read_json(path)
                if command.get("session") != self.session:
                    continue
                self.apply(command)
            except (ValueError, RuntimeError, OSError, KeyError, TypeError) as error:
                self.paused = True
                self.status = f"Command rejected: {error}"
            finally:
                path.unlink(missing_ok=True)

    def update(self, now=None):
        self.receive()
        if now is not None:
            if self._next_decision_wall is None or self._pacing_speed != self.speed:
                self._next_decision_wall = now
                self._pacing_speed = self.speed
            if self.paused and not self.single:
                self._next_decision_wall = now
            elif not self.single and now < self._next_decision_wall:
                return False
        if self.single or not self.paused:
            self.last_advance_start_event = self.runtime.next_event
            self.runtime.advance()
            self.single = max(0, self.single - 1)
            if now is not None:
                self._next_decision_wall = max(
                    self._next_decision_wall
                    + self.runtime.decision_duration / self.speed,
                    now - 0.25,
                )
            return True
        return False

    def write_interventions(self, path):
        write_json(
            path,
            {
                "schema_version": 1,
                "session": self.session,
                "recorded_at_unix": time.time(),
                "interventions": self.interventions,
            },
        )
