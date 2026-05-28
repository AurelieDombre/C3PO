from pathlib import Path
import sys

from api.app_paths import ensure_app_data_dir


def ensure_backend_stdio():
    if sys.stdout is not None and sys.stderr is not None:
        return None

    log_dir = ensure_app_data_dir() / "logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    log_file = open(log_dir / "backend.log", "a", encoding="utf-8", buffering=1)

    if sys.stdout is None:
        sys.stdout = log_file
    if sys.stderr is None:
        sys.stderr = log_file

    return log_file
