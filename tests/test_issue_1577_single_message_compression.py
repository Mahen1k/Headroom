"""
Test case for issue #1577: single message compression returns router:noop

Issue: https://github.com/headroomlabs-ai/headroom/issues/1577

The problem was that when compressing a single user message with 
compress_user_messages=True, if the compression failed (ratio >= min_ratio),
no transform marker was added to transforms_applied, resulting in router:noop.

This test verifies that the fix properly adds router:skipped:ratio_too_high 
marker when compression fails.
"""

import pytest
from headroom import compress, CompressConfig


def test_single_message_compression_with_transform_markers():
    """
    Test that single message compression adds proper transform markers
    even when compression fails (ratio too high).
    
    This reproduces the issue where:
    1. compress_user_messages=True
    2. Single user message with large content
    3. Kompress disabled or unavailable
    4. Expected: transforms_applied should contain router:skipped:ratio_too_high
    5. Bug: transforms_applied was empty, returning router:noop
    """
    # Create a large text message (simulating tool output)
    large_text = "This is a test message with lots of repeated content. " * 100
    
    messages = [
        {"role": "user", "content": large_text}
    ]
    
    # Configure compression to skip user messages (default behavior)
    config = CompressConfig(
        compress_user_messages=True,
        target_ratio=0.85,  # This is the min_ratio threshold
        protect_recent=0,   # Don't protect recent messages
    )
    
    # Compress with Kompress disabled to trigger the failure path
    result = compress(
        messages,
        config=config,
        kompress_model="disabled"  # This will cause compression to fail
    )
    
    # Verify that transforms_applied contains a marker (not router:noop)
    assert result.transforms_applied != ["router:noop"], \
        "Expected transform marker, but got router:noop"
    
    # Verify that the marker indicates compression was skipped
    assert any("router:skipped" in marker for marker in result.transforms_applied), \
        f"Expected router:skipped marker, but got: {result.transforms_applied}"
    
    # Verify compression ratio is reported correctly
    assert result.compression_ratio == 1.0, \
        "Expected compression_ratio of 1.0 (no compression)"
    
    print(f"✓ Test passed! transforms_applied: {result.transforms_applied}")


def test_multiple_messages_with_mixed_results():
    """
    Test that multiple messages get proper markers even when some fail.
    
    This verifies that the fix doesn't break the normal flow where
    some messages compress successfully and others fail.
    """
    messages = [
        {"role": "user", "content": "Short message"},
        {"role": "assistant", "content": "A" * 1000},  # Large content
        {"role": "user", "content": "Another short message"},
    ]
    
    config = CompressConfig(
        compress_user_messages=True,
        protect_recent=0,
    )
    
    result = compress(messages, config=config)
    
    # Should have some transforms applied
    assert result.transforms_applied != ["router:noop"], \
        "Expected some transforms to be applied"
    
    print(f"✓ Test passed! transforms_applied: {result.transforms_applied}")


if __name__ == "__main__":
    test_single_message_compression_with_transform_markers()
    test_multiple_messages_with_mixed_results()
    print("\n✅ All tests passed!")
