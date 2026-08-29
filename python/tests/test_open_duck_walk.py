import hashlib
import runpy
from pathlib import Path

import pytest


ROOT = Path(__file__).resolve().parents[2]
WALK = runpy.run_path(ROOT / "scripts/run-open-duck-walk")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def test_frozen_input_validation_rejects_modified_bundle_file(tmp_path):
    bundle = tmp_path / "bundle"
    bundle.mkdir()
    case = tmp_path / "case.md"
    case.write_text("frozen case")
    checkpoint = bundle / "policy.onnx"
    checkpoint.write_bytes(b"approved checkpoint")
    asset = bundle / "asset.xml"
    asset.write_text("original")
    (bundle / "PROVENANCE.toml").write_text(
        f'''[[source]]
sha256 = "{digest(checkpoint)}"

[files]
"policy.onnx" = "{digest(checkpoint)}"
"asset.xml" = "{digest(asset)}"
'''
    )
    validate = WALK["validate_frozen_inputs"]
    validate.__globals__["BUNDLE"] = bundle
    validate.__globals__["FROZEN_CASE_SHA256"] = digest(case)
    validate(case, checkpoint, None)

    asset.write_text("modified")
    with pytest.raises(SystemExit, match="provenance checksum mismatch: asset.xml"):
        validate(case, checkpoint, None)


def test_frozen_acceptance_reports_failed_thresholds():
    run = {
        "physics_steps": 4000,
        "forward_displacement_m": 0.09,
        "min_root_height_m": 0.07,
        "max_abs_roll_rad": 0.9,
        "max_abs_pitch_rad": 0.9,
        "non_foot_collision_count": 1,
    }
    with pytest.raises(SystemExit, match="forward displacement, root height, roll, pitch, non-foot collision"):
        WALK["validate_frozen_run"](run)
