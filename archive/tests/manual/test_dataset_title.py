"""Test script to verify dataset title functionality."""

import re


def format_filename(filename: str) -> str:
    """
    Format filename for display as title.

    Args:
        filename: Original filename

    Returns:
        Formatted title string
    """
    # Clean filename: remove extension, convert to uppercase, remove special characters
    clean_name = filename
    # Remove common extensions
    clean_name = re.sub(r"\.(csv|json)$", "", clean_name, flags=re.IGNORECASE)
    # Replace underscores and dashes with spaces
    clean_name = clean_name.replace("_", " ").replace("-", " ")
    # Remove special characters, keep only alphanumeric and spaces
    clean_name = re.sub(r"[^A-Za-z0-9\s]", "", clean_name)
    # Convert to uppercase
    clean_name = clean_name.upper().strip()

    return clean_name


# Test cases
test_files = [
    "schedule.json",
    "schedule.json",
    "possible_periods.json",
]

print("Testing filename formatting:")
print("=" * 60)
for filename in test_files:
    formatted = format_filename(filename)
    print(f"{filename:35s} -> {formatted}")
