"""Append `.await` to repository calls inside the application services."""

import io
import os

ROOT = r"C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\ekpi-platform"
TARGETS = [
    os.path.join(ROOT, "crates", "application", "src", "governance.rs"),
    os.path.join(ROOT, "crates", "application", "src", "authentication.rs"),
]

METHODS = (
    "record_project",
    "link_evidence",
    "review_evidence",
    "save_workflow",
    "publish_workflow",
    "create_quality_case",
    "update_quality_case",
    "record_quality_inspection",
    "record_gate_decision",
)


def find_close(text, open_index):
    depth = 0
    index = open_index
    in_string = False
    escaped = False
    while index < len(text):
        char = text[index]
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
        else:
            if char == '"':
                in_string = True
            elif char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
                if depth == 0:
                    return index
        index += 1
    raise ValueError("unbalanced parenthesis")


def patch(text):
    changed = True
    while changed:
        changed = False
        for method in METHODS:
            marker = "self.repository." + method + "("
            position = text.find(marker)
            if position < 0:
                continue
            open_index = position + len(marker) - 1
            close = find_close(text, open_index)
            following = text[close + 1 : close + 7]
            if following.startswith(".await"):
                continue
            text = text[: close + 1] + ".await" + text[close + 1 :]
            changed = True
            break
    return text


for path in TARGETS:
    with io.open(path, "r", encoding="utf-8", newline="") as handle:
        text = handle.read()
    with io.open(path, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(patch(text))
    print("patched", os.path.basename(path))
