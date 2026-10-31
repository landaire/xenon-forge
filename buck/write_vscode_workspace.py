#!/usr/bin/env python3
"""Write a VS Code workspace for building and debugging one Xbox 360 title."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def absolute(path: str) -> str:
    value = Path(path)
    if not value.is_absolute():
        value = Path.cwd() / value
    return value.resolve().as_posix()


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--elf", required=True)
    parser.add_argument("--gdb-commands", required=True)
    parser.add_argument("--gdb", required=True)
    parser.add_argument("--listen", required=True)
    parser.add_argument("--architecture", required=True)
    parser.add_argument("--rust-project")
    return parser.parse_args()


def main() -> None:
    arguments = parse_arguments()
    root = Path.cwd().resolve().as_posix()
    task_name = "Xbox 360: deploy and wait for debugger"
    settings: dict[str, object] = {}
    recommendations = ["ms-vscode.cpptools"]
    if arguments.rust_project:
        settings["rust-analyzer.linkedProjects"] = [
            absolute(arguments.rust_project)
        ]
        settings["rust-analyzer.checkOnSave"] = False
        recommendations.append("rust-lang.rust-analyzer")

    workspace = {
        "folders": [{"path": root}],
        "settings": settings,
        "extensions": {"recommendations": recommendations},
        "tasks": {
            "version": "2.0.0",
            "tasks": [
                {
                    "label": task_name,
                    "type": "process",
                    "command": f"{root}/bin/buck2",
                    "windows": {"command": f"{root}/bin/buck2.cmd"},
                    "args": [
                        "run",
                        f"{arguments.target}[debug-server]",
                    ],
                    "isBackground": True,
                    "problemMatcher": [
                        {
                            "pattern": [{"regexp": "^(?!)$"}],
                            "background": {
                                "activeOnStart": True,
                                "beginsPattern": "^starting Xbox 360 debugger$",
                                "endsPattern": "^Xbox 360 debugger ready at ",
                            },
                        }
                    ],
                    "presentation": {
                        "reveal": "always",
                        "panel": "dedicated",
                    },
                }
            ],
        },
        "launch": {
            "version": "0.2.0",
            "configurations": [
                {
                    "name": "Debug Xbox 360 title",
                    "type": "cppdbg",
                    "request": "launch",
                    "program": absolute(arguments.elf),
                    "cwd": root,
                    "MIMode": "gdb",
                    "miDebuggerPath": arguments.gdb,
                    "miDebuggerServerAddress": arguments.listen,
                    "preLaunchTask": task_name,
                    "externalConsole": False,
                    "setupCommands": [
                        {"text": "set pagination off"},
                        {"text": "set confirm off"},
                        {
                            "text": "set architecture " + arguments.architecture,
                        },
                        {"text": "set endian big"},
                        {
                            "text": "source \""
                            + absolute(arguments.gdb_commands)
                            + "\"",
                        },
                    ],
                }
            ],
        },
    }
    arguments.output.write_text(json.dumps(workspace, indent=2) + "\n")


if __name__ == "__main__":
    main()
