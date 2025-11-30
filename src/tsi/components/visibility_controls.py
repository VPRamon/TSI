"""Deprecated compatibility shim for visibility controls.

This module used to contain the visibility controls implementation. The
canonical implementation now lives in
`tsi.components.visibility.visibility_controls`. This file re-exports that
implementation for compatibility and emits a deprecation warning.
"""

from __future__ import annotations

import warnings

warnings.warn(
    "Importing `tsi.components.visibility_controls` is deprecated; import from "
    "`tsi.components.visibility.visibility_controls` instead.",
    DeprecationWarning,
)

# Re-export the canonical implementation
from tsi.components.visibility.visibility_controls import *  # noqa: F401,F403
