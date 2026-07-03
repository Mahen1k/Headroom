from __future__ import annotations

from types import SimpleNamespace

import pytest

from headroom.proxy import savings_tracker as savings_tracker_module


def test_estimate_cache_savings_usd_uses_discounted_cache_read_price(monkeypatch) -> None:
    fake_litellm = SimpleNamespace(
        model_cost={
            "gpt-4o": {
                "input_cost_per_token": 0.002,
                "cache_read_input_token_cost": 0.001,
            }
        }
    )
    monkeypatch.setattr(savings_tracker_module, "LITELLM_AVAILABLE", True)
    monkeypatch.setattr(savings_tracker_module, "litellm", fake_litellm)

    assert savings_tracker_module._estimate_cache_savings_usd("gpt-4o", 100) == pytest.approx(
        0.1
    )


def test_estimate_cache_savings_usd_handles_missing_pricing(monkeypatch) -> None:
    fake_litellm = SimpleNamespace(model_cost={})
    monkeypatch.setattr(savings_tracker_module, "LITELLM_AVAILABLE", True)
    monkeypatch.setattr(savings_tracker_module, "litellm", fake_litellm)

    assert savings_tracker_module._estimate_cache_savings_usd("gpt-4o", 100) == 0.0

    monkeypatch.setattr(savings_tracker_module, "LITELLM_AVAILABLE", False)
    assert savings_tracker_module._estimate_cache_savings_usd("gpt-4o", 100) == 0.0
