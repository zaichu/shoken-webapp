#!/usr/bin/env python3
import json
import re
import sys


def extract_command(payload: dict) -> str:
    tool_input = payload.get("tool_input")
    if isinstance(tool_input, dict):
        for key in ("command", "cmd"):
            value = tool_input.get(key)
            if isinstance(value, str):
                return value
    tool_result = payload.get("tool_result")
    if isinstance(tool_result, str):
        return tool_result
    return ""


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        return 0

    if payload.get("tool_name") != "Bash":
        return 0

    command_text = extract_command(payload)
    if not command_text:
        return 0

    if not re.search(r"\bgit push\b|\bgh pr create\b|\bgh pr edit\b|\bgh pr merge\b", command_text):
        return 0

    message = {
        "systemMessage": (
            "コード変更を push / PR 更新したので、レビュー運用では Codex CLI の再レビューを必ず実行してください。 "
            "必要なら /review-ops:pr-review-loop を使ってください。"
        )
    }
    print(json.dumps(message, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
