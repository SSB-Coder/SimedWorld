import math

from ..habitat import Habitat


def inspect_consumers(snapshot, perception, feeding):
    organisms = snapshot["organisms"]
    producers = [r for r in organisms if r["organism"]["alive"] and r["organism"]["genome"]["photosynthetic"]]
    rows = []
    for record in organisms:
        organism = record["organism"]
        if not organism["genome"]["can_ingest"]:
            continue
        p = record["position"]
        nearest = min((math.hypot(p["x"] - r["position"]["x"], p["y"] - r["position"]["y"]) for r in producers), default=None)
        rows.append({"id": organism["id"], "alive": organism["alive"], "age": organism["age"],
                     "death_cause": organism["death_cause"], "tissue_nutrient": organism["tissue"]["nutrient"],
                     "reserve_energy": organism["reserves"]["energy"], "nearest_producer_distance": nearest,
                     "food_in_sensing_radius": nearest is not None and nearest <= perception,
                     "food_in_feeding_radius": nearest is not None and nearest <= feeding})
    return rows


def diagnose_legacy(rules, config, ticks=3000, sample_every=300):
    habitat = Habitat(rules, config)
    perception = config["ecology"]["perception_radius"]
    feeding = config["ecology"]["feeding_radius"]
    last_alive, death_records, samples = {}, {}, []
    duration = rules["physics_dt"] * rules["ecology_interval_ticks"]
    while habitat.tick < ticks:
        before = inspect_consumers(habitat.snapshot(), perception, feeding)
        last_alive.update({r["id"]: r for r in before if r["alive"]})
        habitat.advance(min(rules["ecology_interval_ticks"], ticks - habitat.tick))
        after = inspect_consumers(habitat.snapshot(), perception, feeding)
        for row in after:
            if not row["alive"] and row["id"] not in death_records:
                death_records[row["id"]] = {"tick": habitat.tick, "id": row["id"], "cause": row["death_cause"],
                                            "previous_window": last_alive.get(row["id"])}
        if habitat.tick % sample_every == 0 or habitat.tick == ticks:
            summary = habitat.summary()
            living = [r for r in after if r["alive"]]
            samples.append({"tick": habitat.tick, "producers": summary["producer_count"], "consumers": len(living),
                            "mean_consumer_energy": sum(r["reserve_energy"] for r in living) / max(1, len(living)),
                            "consumers_with_food_in_range": sum(r["food_in_sensing_radius"] for r in living),
                            "feeding_events": summary["ecology"]["feeding_events"]})
    summary = habitat.summary()
    ecology = summary["ecology"]
    return {"schema_version": 1, "scope": "Diagnostic scripted consumer preset; observer-only global distances are not neural inputs.",
            "samples": samples, "consumer_deaths": list(death_records.values()), "final": summary, "audit": habitat.audit(),
            "energy_flows": {"movement_expenditure": ecology["distance_moved"] * config["ecology"]["movement_energy_per_distance"],
                             "digestion_expenditure": ecology["ingested_nutrient"] * rules["digestion_energy_per_ingested_nutrient"],
                             "assimilated_food_energy": ecology["ingested_nutrient"] * rules["tissue_energy_per_nutrient"] * rules["energy_assimilation_fraction"],
                             "sensing_expenditure_upper_bound": ecology["consumers_processed"] * duration * rules["sensor_energy_per_time"]},
            "light": {"renewed_every_window": True, "per_organism_absorption_limit_per_time": rules["max_absorbed_light_per_time"],
                      "incident": summary["incident_light"], "captured": summary["absorbed_light"]},
            "interpretation": "Report resource expenditure, encounters, and last living states before calibrating. Last-window food distance alone does not establish an extinction cause; positive captured-light totals do not imply accessible consumer energy."}
