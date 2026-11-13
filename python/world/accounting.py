import json

from ._native import ResourceLedger as NativeResourceLedger


class ResourceLedger:
    def __init__(self, rules: dict, initial: dict):
        self._native = NativeResourceLedger(json.dumps(rules, allow_nan=False),
                                             json.dumps(initial, allow_nan=False))

    def apply(self, operation: str, **arguments):
        if "kind" in arguments:
            raise ValueError("kind is reserved for the operation name")
        command = {"schema_version": 1, "operation": {"kind": operation, **arguments}}
        self._native.apply_json(json.dumps(command, allow_nan=False))

    def audit(self) -> dict:
        return json.loads(self._native.audit_json())

    def summary(self, label: str = "state") -> dict:
        return json.loads(self._native.summary_json(label))

    def snapshot(self) -> dict:
        return json.loads(self._native.snapshot_json())
