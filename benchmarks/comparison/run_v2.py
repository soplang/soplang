"""Launch the unmodified file-execution pipeline from an exported v2 revision."""

from pathlib import Path
import sys

sys.path.insert(0, str(Path(sys.argv[1]).resolve()))

from src.runtime.main import run_soplang_file

raise SystemExit(run_soplang_file(sys.argv[2]))
