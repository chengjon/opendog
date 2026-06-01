from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PLANNING_DIR = ROOT / ".planning"
FUNCTION_TREE_FILE = ROOT / "FUNCTION_TREE.md"
REQUIREMENTS_FILE = PLANNING_DIR / "REQUIREMENTS.md"
ROADMAP_FILE = PLANNING_DIR / "ROADMAP.md"
TASK_CARD_DIR = PLANNING_DIR / "task-cards"
