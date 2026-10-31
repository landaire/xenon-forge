#!/usr/bin/env python3
"""Deploy and launch an Xbox 360 title through xeedee."""

from __future__ import annotations

import argparse
import errno
import os
from pathlib import Path
import socket
import subprocess
import sys
import threading
import time


def project_root(start: Path) -> Path:
    for directory in (start, *start.parents):
        if (directory / ".buckroot").is_file():
            return directory
    return start


def remote_directory(configured: str | None, xex: Path) -> str:
    root = configured or rf"DEVKIT:\{project_root(Path.cwd()).name}"
    root = root.replace("/", "\\").rstrip("\\")
    return root + "\\" + xex.stem


def remote_parts(path: str) -> tuple[str, list[str]]:
    drive, separator, rest = path.partition("\\")
    if not separator or not drive.endswith(":"):
        raise ValueError(
            f"remote directory {path!r} must be an Xbox drive path such as "
            r"DEVKIT:\project"
        )
    parts = [part for part in rest.split("\\") if part]
    if not parts:
        raise ValueError(f"remote directory {path!r} must be below the drive root")
    return drive, parts


def xeedee_command(tool: str, host: str, *arguments: str) -> list[str]:
    return [tool, "--host", host, *arguments]


def make_remote_directories(tool: str, host: str, directory: str) -> None:
    drive, parts = remote_parts(directory)
    current = drive + "\\"
    for part in parts:
        current += part
        status = subprocess.run(
            xeedee_command(tool, host, "file", "stat", current),
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
        if status.returncode != 0:
            subprocess.run(
                xeedee_command(tool, host, "file", "mkdir", current),
                check=True,
            )
        current += "\\"


def deploy(tool: str, host: str, directory: str, xex: Path) -> str:
    make_remote_directories(tool, host, directory)
    remote = directory + "\\" + xex.name
    subprocess.run(
        xeedee_command(tool, host, "file", "put", os.fspath(xex), remote),
        check=True,
    )
    return remote


def launch(
    tool: str,
    host: str,
    directory: str,
    remote: str,
    title_arguments: list[str],
) -> None:
    command = xeedee_command(
        tool,
        host,
        "magicboot",
        "--title",
        remote,
        "--directory",
        directory,
    )
    command.extend(["--cmdline", subprocess.list2cmdline(title_arguments)])
    subprocess.run(command, check=True)


def listen_address(value: str) -> tuple[str, int]:
    if value.startswith("["):
        host, separator, port = value[1:].partition("]:")
    else:
        host, separator, port = value.rpartition(":")
    if not separator or not host:
        raise ValueError(f"debugger listen address {value!r} must be HOST:PORT")
    return host, int(port)


def port_is_bound(host: str, port: int) -> bool:
    for family, kind, protocol, _, address in socket.getaddrinfo(
        host, port, type=socket.SOCK_STREAM
    ):
        with socket.socket(family, kind, protocol) as listener:
            try:
                listener.bind(address)
            except OSError as error:
                if error.errno in (errno.EACCES, errno.EADDRINUSE):
                    return True
                raise
    return False


def forward_debugger_output(
    process: subprocess.Popen[str],
    ready: threading.Event,
    title_loaded: threading.Event,
) -> None:
    assert process.stdout is not None
    for line in process.stdout:
        sys.stdout.write(line)
        sys.stdout.flush()
        if "waiting for a GDB client on " in line:
            ready.set()
        if "stopped at the load of " in line:
            title_loaded.set()


def wait_for_debugger_event(
    process: subprocess.Popen[str],
    event: threading.Event,
    description: str,
    timeout: float = 30.0,
) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        status = process.poll()
        if status is not None:
            raise RuntimeError(
                f"openxbdm-kd exited before {description} (status {status})"
            )
        if event.wait(0.05):
            return
    raise RuntimeError(f"openxbdm-kd did not {description} within {timeout:g} seconds")


def terminate(process: subprocess.Popen[str]) -> None:
    if process.poll() is not None:
        return
    process.terminate()
    try:
        process.wait(timeout=3)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


def debug(
    kd_tool: str,
    kd_target: str | None,
    gdb: str | None,
    gdb_commands: Path | None,
    listen: str,
    registers: str,
    xbdm_tool: str,
    xbdm_host: str,
    directory: str,
    remote: str,
    title_arguments: list[str],
) -> int:
    host, port = listen_address(listen)
    if port_is_bound(host, port):
        raise RuntimeError(f"debugger listen address {listen} is already in use")

    kd_command = [kd_tool]
    if kd_target:
        kd_command.extend(["--target", kd_target])
    else:
        kd_command.append("--no-timeout")
    kd_command.extend(
        [
            "gdbserver",
            "--listen",
            listen,
            "--registers",
            registers,
            "--stop-on-load",
            remote.rsplit("\\", 1)[-1],
        ]
    )

    ready = threading.Event()
    title_loaded = threading.Event()
    server = subprocess.Popen(
        kd_command,
        stdout=subprocess.PIPE,
        text=True,
        bufsize=1,
    )
    output = threading.Thread(
        target=forward_debugger_output,
        args=(server, ready, title_loaded),
        daemon=True,
    )
    output.start()
    try:
        wait_for_debugger_event(server, ready, "become ready")
        launch(xbdm_tool, xbdm_host, directory, remote, title_arguments)
        wait_for_debugger_event(server, title_loaded, "stop at the title load")
        if gdb is None:
            print(f"Xbox 360 debugger ready at {listen}", flush=True)
            return server.wait()

        assert gdb_commands is not None
        architecture = "powerpc:common64" if registers == "64" else "powerpc:common"
        debugger = subprocess.run(
            [
                gdb,
                "-q",
                "-nx",
                "-ex",
                "set pagination off",
                "-ex",
                "set confirm off",
                "-ex",
                "set architecture " + architecture,
                "-ex",
                "set endian big",
                "-x",
                os.fspath(gdb_commands),
                "-ex",
                f"target remote {listen}",
            ],
            check=False,
        )
        return debugger.returncode
    finally:
        terminate(server)


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", help="XBDM console host from xbox360.console_xbdm")
    parser.add_argument("--xbdm-tool", default="xeedee")
    parser.add_argument("--directory")
    debugger = parser.add_mutually_exclusive_group()
    debugger.add_argument("--debug", action="store_true")
    debugger.add_argument("--debug-server", action="store_true")
    parser.add_argument("--kd-tool", default="openxbdm-kd")
    parser.add_argument("--kd-target")
    parser.add_argument("--gdb", default="gdb")
    parser.add_argument("--gdb-commands", type=Path)
    parser.add_argument("--listen", default="127.0.0.1:2159")
    parser.add_argument("--registers", choices=("32", "ilp32", "64"), default="32")
    parser.add_argument("xex", type=Path)
    parser.add_argument("title_arguments", nargs=argparse.REMAINDER)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    if not arguments.host:
        raise SystemExit(
            "run_title.py: no console configured; add console_xbdm = HOST to the "
            "[xbox360] section of .buckconfig.local"
        )
    if arguments.debug and arguments.gdb_commands is None:
        raise SystemExit("run_title.py: --debug requires --gdb-commands")
    if arguments.debug or arguments.debug_server:
        print("starting Xbox 360 debugger", flush=True)

    directory = remote_directory(arguments.directory, arguments.xex)
    remote = deploy(arguments.xbdm_tool, arguments.host, directory, arguments.xex)
    if not (arguments.debug or arguments.debug_server):
        launch(
            arguments.xbdm_tool,
            arguments.host,
            directory,
            remote,
            arguments.title_arguments,
        )
        return 0
    return debug(
        arguments.kd_tool,
        arguments.kd_target,
        None if arguments.debug_server else arguments.gdb,
        arguments.gdb_commands,
        arguments.listen,
        arguments.registers,
        arguments.xbdm_tool,
        arguments.host,
        directory,
        remote,
        arguments.title_arguments,
    )


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, subprocess.SubprocessError, RuntimeError, ValueError) as error:
        raise SystemExit(f"run_title.py: {error}") from error
