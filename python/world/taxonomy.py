"""Descriptive genetic clusters; these labels never affect ecological selection."""

import math


def genetic_distance(left, right):
    a, b = left["body"], right["body"]
    result = float(a["diet"] != b["diet"])
    result += float(left["photosynthetic"] != right["photosynthetic"])
    for name in ["speed", "perception", "learning_rate"]:
        result += abs(a["circle"][name] - b["circle"][name]) / 3.0
    for name in [
        "camouflage",
        "pigment",
        "armor",
        "filament",
        "communication",
        "burst",
        "offense",
        "sociality",
    ]:
        result += (
            abs(
                a["circle"].get("adaptations", {}).get(name, 0)
                - b["circle"].get("adaptations", {}).get(name, 0)
            )
            / 6.0
        )
    result += abs(math.log(a["density"] / b["density"])) / 5.0
    result += abs(a["modules"][0]["radius"] - b["modules"][0]["radius"])
    ga, gb = left.get("brain", {}), right.get("brain", {})
    na, nb = len((ga or {}).get("nodes", [])), len((gb or {}).get("nodes", []))
    result += abs(na - nb) / max(1, na, nb)
    return result


def classify(organisms, threshold=0.3):
    if not math.isfinite(threshold) or threshold <= 0:
        raise ValueError("invalid cluster threshold")
    representatives = []
    clusters = {}
    roots = {}
    for o in sorted(organisms, key=lambda o: o["id"]):
        roots[o["id"]] = roots.get(o.get("parent_id"), o["id"])
        if not o["alive"] or not o["genome"].get("body", {}).get("circle"):
            continue
        label = next(
            (
                i
                for i, g in enumerate(representatives)
                if genetic_distance(o["genome"], g) <= threshold
            ),
            None,
        )
        if label is None:
            label = len(representatives)
            representatives.append(o["genome"])
        clusters[o["id"]] = {"cluster": label + 1, "founder_lineage": roots[o["id"]]}
    return {
        "threshold": threshold,
        "cluster_count": len(representatives),
        "members": clusters,
        "interpretation": "descriptive genome similarity; no mating barriers or species quotas",
    }
