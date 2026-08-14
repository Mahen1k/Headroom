"""
Microgreens salability tool.

Tracks grow batches and decides whether they are ready to sell:
  1. Harvest readiness  - is the batch inside its variety's harvest window?
  2. Quality grading     - does it pass mold/uniformity/stem/root checks?
  3. Listing generation  - produce a sellable listing for batches that pass both.

Usage:
    python salability.py add-batch --variety sunflower --sow-date 2026-08-01 \
        --quantity 12 --price 4.50

    python salability.py inspect --id 1 --height 7.5 --mold no \
        --color-uniform yes --stem-ok yes --root-mat-ok yes

    python salability.py list-salable
"""

from __future__ import annotations

import argparse
import json
from dataclasses import asdict, dataclass
from datetime import date, datetime
from pathlib import Path

DATA_FILE = Path(__file__).parent / "batches.json"

# (min_days, max_days, min_height_cm, max_height_cm) for each variety's harvest window.
MICROGREEN_VARIETIES: dict[str, tuple[int, int, float, float]] = {
    "sunflower": (8, 12, 6.0, 10.0),
    "pea": (8, 14, 5.0, 12.0),
    "radish": (6, 10, 4.0, 8.0),
    "broccoli": (7, 12, 3.0, 7.0),
    "arugula": (6, 10, 3.0, 6.0),
    "kale": (8, 12, 3.0, 6.0),
    "cilantro": (14, 21, 4.0, 8.0),
    "basil": (14, 21, 3.0, 6.0),
}


@dataclass
class QualityCheck:
    height_cm: float
    mold: bool
    color_uniform: bool
    stem_ok: bool
    root_mat_ok: bool


@dataclass
class Batch:
    id: int
    variety: str
    sow_date: str  # ISO date
    quantity_trays: int
    price_per_tray: float
    quality: QualityCheck | None = None

    def days_since_sowing(self, today: date | None = None) -> int:
        today = today or date.today()
        return (today - date.fromisoformat(self.sow_date)).days


def load_batches() -> list[Batch]:
    if not DATA_FILE.exists():
        return []
    raw = json.loads(DATA_FILE.read_text())
    batches = []
    for item in raw:
        quality = item.pop("quality", None)
        batch = Batch(**item, quality=QualityCheck(**quality) if quality else None)
        batches.append(batch)
    return batches


def save_batches(batches: list[Batch]) -> None:
    DATA_FILE.write_text(json.dumps([asdict(b) for b in batches], indent=2))


def next_batch_id(batches: list[Batch]) -> int:
    return max((b.id for b in batches), default=0) + 1


def harvest_readiness(batch: Batch, today: date | None = None) -> tuple[bool, list[str]]:
    """Check whether a batch is within its variety's harvest window."""
    if batch.variety not in MICROGREEN_VARIETIES:
        return False, [f"unknown variety '{batch.variety}'"]

    min_days, max_days, min_height, max_height = MICROGREEN_VARIETIES[batch.variety]
    reasons = []
    days = batch.days_since_sowing(today)

    if days < min_days:
        reasons.append(f"only {days} days since sowing, needs at least {min_days}")
    elif days > max_days:
        reasons.append(f"{days} days since sowing exceeds max {max_days} (overgrown)")

    if batch.quality is not None:
        h = batch.quality.height_cm
        if h < min_height:
            reasons.append(f"height {h}cm below minimum {min_height}cm")
        elif h > max_height:
            reasons.append(f"height {h}cm above maximum {max_height}cm")

    return len(reasons) == 0, reasons


def quality_grade(quality: QualityCheck) -> tuple[str, list[str]]:
    """Grade a batch as 'A', 'B', or 'reject' based on its quality check."""
    reasons = []

    if quality.mold:
        return "reject", ["mold detected"]

    if not quality.stem_ok:
        reasons.append("stems too short/leggy")
    if not quality.root_mat_ok:
        reasons.append("root mat not fully established")
    if not quality.color_uniform:
        reasons.append("uneven coloring")

    if not reasons:
        return "A", []
    if len(reasons) == 1:
        return "B", reasons
    return "reject", reasons


def is_salable(batch: Batch, today: date | None = None) -> tuple[bool, str, list[str]]:
    """Return (salable, grade, reasons) combining readiness and quality."""
    ready, ready_reasons = harvest_readiness(batch, today)
    if batch.quality is None:
        return False, "unrated", ready_reasons + ["no quality check recorded"]

    grade, grade_reasons = quality_grade(batch.quality)
    reasons = ready_reasons + grade_reasons
    salable = ready and grade in ("A", "B")
    return salable, grade, reasons


def build_listing(batch: Batch) -> dict:
    salable, grade, reasons = is_salable(batch)
    if not salable:
        raise ValueError(f"batch {batch.id} is not salable: {', '.join(reasons)}")

    variety_title = batch.variety.replace("_", " ").title()
    return {
        "title": f"Fresh {variety_title} Microgreens - Grade {grade}",
        "description": (
            f"{batch.quantity_trays} tray(s) of {variety_title} microgreens, "
            f"harvested {batch.days_since_sowing()} days after sowing. "
            f"Grade {grade}."
        ),
        "quantity": batch.quantity_trays,
        "unit_price": batch.price_per_tray,
        "total_price": round(batch.quantity_trays * batch.price_per_tray, 2),
        "grade": grade,
    }


def _parse_bool(value: str) -> bool:
    return value.strip().lower() in ("yes", "y", "true", "1")


def main() -> None:
    parser = argparse.ArgumentParser(description="Microgreens salability tool")
    sub = parser.add_subparsers(dest="command", required=True)

    add_p = sub.add_parser("add-batch", help="Register a new grow batch")
    add_p.add_argument("--variety", required=True, choices=sorted(MICROGREEN_VARIETIES))
    add_p.add_argument("--sow-date", required=True, help="YYYY-MM-DD")
    add_p.add_argument("--quantity", type=int, required=True, help="number of trays")
    add_p.add_argument("--price", type=float, required=True, help="price per tray")

    insp_p = sub.add_parser("inspect", help="Record a quality check for a batch")
    insp_p.add_argument("--id", type=int, required=True)
    insp_p.add_argument("--height", type=float, required=True, help="height in cm")
    insp_p.add_argument("--mold", required=True, help="yes/no")
    insp_p.add_argument("--color-uniform", required=True, help="yes/no")
    insp_p.add_argument("--stem-ok", required=True, help="yes/no")
    insp_p.add_argument("--root-mat-ok", required=True, help="yes/no")

    sub.add_parser("list-salable", help="List batches that are ready to sell")
    sub.add_parser("list-batches", help="List all tracked batches")

    args = parser.parse_args()
    batches = load_batches()

    if args.command == "add-batch":
        datetime.fromisoformat(args.sow_date)  # validate format
        batch = Batch(
            id=next_batch_id(batches),
            variety=args.variety,
            sow_date=args.sow_date,
            quantity_trays=args.quantity,
            price_per_tray=args.price,
        )
        batches.append(batch)
        save_batches(batches)
        print(f"Added batch {batch.id}: {batch.variety} sown {batch.sow_date}")

    elif args.command == "inspect":
        batch = next((b for b in batches if b.id == args.id), None)
        if batch is None:
            parser.error(f"no batch with id {args.id}")
        batch.quality = QualityCheck(
            height_cm=args.height,
            mold=_parse_bool(args.mold),
            color_uniform=_parse_bool(args.color_uniform),
            stem_ok=_parse_bool(args.stem_ok),
            root_mat_ok=_parse_bool(args.root_mat_ok),
        )
        save_batches(batches)

        salable, grade, reasons = is_salable(batch)
        print(f"Batch {batch.id} ({batch.variety}): grade={grade}, salable={salable}")
        for reason in reasons:
            print(f"  - {reason}")
        if salable:
            listing = build_listing(batch)
            print("\nListing:")
            print(json.dumps(listing, indent=2))

    elif args.command == "list-salable":
        found = False
        for batch in batches:
            salable, grade, _ = is_salable(batch)
            if salable:
                found = True
                listing = build_listing(batch)
                print(f"#{batch.id} {listing['title']} - ${listing['total_price']}")
        if not found:
            print("No salable batches yet.")

    elif args.command == "list-batches":
        for batch in batches:
            salable, grade, reasons = is_salable(batch)
            status = "SALABLE" if salable else "not ready"
            print(f"#{batch.id} {batch.variety} sown {batch.sow_date} [{status}, grade={grade}]")


if __name__ == "__main__":
    main()
