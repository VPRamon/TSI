"""Utility functions for displaying errors in the UI with proper security practices.

This module provides helpers for displaying errors to users while:
- Hiding sensitive implementation details (database schemas, connection strings, etc.)
- Logging detailed error information for debugging
- Providing consistent, user-friendly error messages
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import streamlit as st

logger = logging.getLogger(__name__)


def display_error(error: Exception, default_message: str = "An error occurred") -> None:
    """Display an error to the user with appropriate security considerations.
    
    This function:
    1. Logs the detailed error with full context for debugging
    2. Shows a user-friendly message that doesn't expose sensitive details
    3. Provides helpful guidance for the user
    
    Args:
        error: The exception that occurred
        default_message: Default message to show if error doesn't have a user message
    """
    import streamlit as st
    from tsi.exceptions import TSIError
    
    # Log detailed error for debugging
    if isinstance(error, TSIError):
        logger.error(
            f"{error.__class__.__name__}: {error.message}",
            exc_info=True,
            extra={"details": error.details}
        )
        # Show user-friendly message
        st.error(error.to_user_message())
    else:
        # Log unexpected errors with full details
        logger.error(f"Unexpected error: {error}", exc_info=True)
        # Show generic message
        st.error(default_message)
    
    # Provide helpful guidance
    st.info("If this problem persists, please check the application logs or contact support.")


def display_data_error(error: Exception) -> None:
    """Display a data loading error with appropriate context.
    
    Args:
        error: The exception that occurred during data loading
    """
    display_error(error, "Failed to load data. Please try again.")


def display_backend_error(error: Exception) -> None:
    """Display a backend service error with appropriate context.
    
    Args:
        error: The exception that occurred in the backend
    """
    display_error(error, "A backend service error occurred. Please try again later.")


__all__ = [
    "display_error",
    "display_data_error", 
    "display_backend_error",
]
