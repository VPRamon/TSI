"""Top-level Streamlit entrypoint for Streamlit Cloud.

Streamlit Cloud looks for `streamlit_app.py` by default. This file imports
the package entrypoint so `from tsi import state` and other package imports
work once the package is installed (editable install via requirements).
"""

from tsi.app import main


if __name__ == "__main__":
    main()
