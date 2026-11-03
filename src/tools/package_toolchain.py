#!/usr/bin/env python3
"""Package the prebuilt Xbox 360 Rust and LLVM toolchain.

`package` archives the toolchain of the host that built it. `release` combines
the archives of every host into the manifest, the Buck2 pin and the DotSlash
descriptors.
"""

import argparse
import gzip
import hashlib
import json
import lzma
import os
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path
from typing import NamedTuple


class Host(NamedTuple):
    # The `prelude//os` and `prelude//cpu` constraints of the host.
    buck_os: str
    buck_cpu: str
    # The DotSlash platform name.
    dotslash: str
    executable_suffix: str
    # The Windows tar.exe of Windows 10 cannot read xz.
    archive_format: str


HOSTS = {
    "x86_64-unknown-linux-gnu": Host("linux", "x86_64", "linux-x86_64", "", "tar.xz"),
    "aarch64-apple-darwin": Host("macos", "arm64", "macos-aarch64", "", "tar.xz"),
    "x86_64-pc-windows-msvc": Host("windows", "x86_64", "windows-x86_64", ".exe", "tar.gz"),
}
RUST_TOOLS = ("rustc", "rustdoc", "clippy-driver")
LLVM_REQUIRED_TOOLS = (
    "clang",
    "clang++",
    "ld.lld",
    "llvm-ar",
    "llvm-objcopy",
    "llvm-objdump",
)
LLVM_OPTIONAL_TOOLS = (
    "FileCheck",
    "clang-cpp",
    "llc",
    "lld",
    "llvm-addr2line",
    "llvm-dwarfdump",
    "llvm-mc",
    "llvm-nm",
    "llvm-ranlib",
    "llvm-readelf",
    "llvm-readobj",
    "llvm-size",
    "llvm-strings",
    "llvm-strip",
    "llvm-symbolizer",
    "opt",
)
INPUT_FIELDS = ("bootstrap_commit", "llvm_commit", "rust_commit", "toolchain_key")


def root_name(host: str) -> str:
    return f"xbox360-toolchain-{host}"


def digest(path: Path, algorithm: str) -> str:
    value = hashlib.new(algorithm)
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


class ToolCopier:
    """Copies tools into bin/, linking files that have the same contents.

    LLVM installs its tool aliases as symlinks, except on Windows, where it
    installs copies. Hard links keep each copy once in the archive.
    """

    def __init__(self, destination: Path, suffix: str) -> None:
        self.destination = destination
        self.suffix = suffix
        self.by_digest: dict[str, Path] = {}
        destination.mkdir(parents=True, exist_ok=True)

    def copy(self, source: Path, name: str, selected: set[str]) -> None:
        destination = self.destination / (name + self.suffix)
        if source.is_symlink():
            target = Path(os.readlink(source))
            if not target.is_absolute() and target.name in selected:
                destination.symlink_to(target.name)
                return
        source = source.resolve()
        content = digest(source, "sha256")
        existing = self.by_digest.get(content)
        if existing is not None:
            os.link(existing, destination)
            return
        shutil.copy2(source, destination)
        self.by_digest[content] = destination

    def copy_set(
        self,
        source: Path,
        required: tuple[str, ...],
        optional: tuple[str, ...] = (),
    ) -> list[str]:
        def present(name: str) -> bool:
            return (source / (name + self.suffix)).exists()

        missing = [name for name in required if not present(name)]
        if missing:
            raise SystemExit(f"{source} is missing required tools: {', '.join(missing)}")

        names = list(required) + [name for name in optional if present(name)]
        selected = {name + self.suffix for name in names}
        for name in names:
            self.copy(source / (name + self.suffix), name, selected)
        return names


def normalize(source_date_epoch: int):
    def apply(info: tarfile.TarInfo) -> tarfile.TarInfo:
        info.mtime = source_date_epoch
        info.uid = info.gid = 0
        info.uname = info.gname = ""
        if info.isdir() or info.mode & 0o111:
            info.mode = 0o755
        else:
            info.mode = 0o644
        return info

    return apply


def write_archive(
    path: Path, root: Path, archive_format: str, source_date_epoch: int, level: int
) -> None:
    """A tar archive of `root` with sorted entries and fixed metadata."""
    paths = [root] + sorted(root.rglob("*"), key=lambda entry: entry.relative_to(root).as_posix())
    apply = normalize(source_date_epoch)
    with path.open("wb") as file:
        if archive_format == "tar.xz":
            stream = lzma.LZMAFile(file, "wb", preset=level)
        else:
            # The gzip header records a time and a file name.
            stream = gzip.GzipFile("", "wb", level, file, mtime=source_date_epoch)
        with stream, tarfile.open(fileobj=stream, mode="w", format=tarfile.PAX_FORMAT) as archive:
            for entry in paths:
                name = (Path(root.name) / entry.relative_to(root)).as_posix()
                # Later links to a file become hard link entries.
                archive.add(entry, arcname=name, recursive=False, filter=apply)


def package(args: argparse.Namespace) -> int:
    host = HOSTS[args.host]
    for directory in (args.rust_dist, args.llvm_dist):
        if not directory.is_dir():
            raise SystemExit(f"not a distribution directory: {directory}")
    if not args.target_spec.is_file():
        raise SystemExit(f"target specification does not exist: {args.target_spec}")
    if not args.dotslash.is_file():
        raise SystemExit(f"DotSlash does not exist: {args.dotslash}")
    if not 0 <= args.compression_level <= 9:
        raise SystemExit("--compression-level must be between 0 and 9")

    toolchain_inputs = read_json(args.toolchain_inputs)
    missing_inputs = [name for name in INPUT_FIELDS if name not in toolchain_inputs]
    if missing_inputs:
        raise SystemExit(f"toolchain inputs are missing: {', '.join(missing_inputs)}")

    args.out_dir.mkdir(parents=True, exist_ok=True)
    root_dir = root_name(args.host)
    archive_name = f"{root_dir}.{host.archive_format}"
    archive_path = args.out_dir / archive_name

    with tempfile.TemporaryDirectory(prefix="xbox360-toolchain-") as temporary:
        root = Path(temporary) / root_dir
        copier = ToolCopier(root / "bin", host.executable_suffix)
        rust_tools = copier.copy_set(args.rust_dist / "bin", RUST_TOOLS)
        llvm_tools = copier.copy_set(args.llvm_dist / "bin", LLVM_REQUIRED_TOOLS, LLVM_OPTIONAL_TOOLS)
        if not (root / "bin" / ("rust-lld" + host.executable_suffix)).exists():
            copier.copy(args.llvm_dist / "bin" / ("ld.lld" + host.executable_suffix), "rust-lld", set())
            llvm_tools.append("rust-lld")

        shutil.copytree(args.rust_dist / "lib" / "rustlib", root / "lib" / "rustlib", symlinks=False)
        shutil.copytree(args.llvm_dist / "lib" / "clang", root / "lib" / "clang", symlinks=False)
        for library in args.runtime_library:
            if not library.is_file():
                raise SystemExit(f"runtime library does not exist: {library}")
            shutil.copy2(library.resolve(), root / "lib" / library.name)

        target_dir = root / "share" / "xbox360" / "targets"
        target_dir.mkdir(parents=True)
        shutil.copy2(args.target_spec, target_dir / args.target_spec.name)

        metadata = {
            **{name: toolchain_inputs[name] for name in INPUT_FIELDS},
            "host": args.host,
            "llvm_tools": sorted(llvm_tools),
            "rust_tools": sorted(rust_tools),
            "source_commit": args.source_commit,
            "target_spec": f"share/xbox360/targets/{args.target_spec.name}",
        }
        write_json(root / "share" / "xbox360" / "toolchain.json", metadata)
        write_archive(
            archive_path,
            root,
            host.archive_format,
            args.source_date_epoch,
            args.compression_level,
        )

    blake3 = subprocess.check_output(
        [str(args.dotslash), "--", "b3sum", str(archive_path)], text=True
    ).strip()
    write_json(
        args.out_dir / f"{root_dir}.json",
        {
            **metadata,
            "archive": {
                "blake3": blake3,
                "format": host.archive_format,
                "name": archive_name,
                "sha256": digest(archive_path, "sha256"),
                "size": archive_path.stat().st_size,
                "strip_prefix": root_dir,
            },
            "toolchain_inputs": toolchain_inputs,
        },
    )
    return 0


def write_dotslash(path: Path, platforms: dict[str, dict]) -> None:
    descriptor = {"name": path.name, "platforms": platforms}
    path.write_text(
        "#!/usr/bin/env dotslash\n\n" + json.dumps(descriptor, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    path.chmod(0o755)


def write_buck_pin(path: Path, manifest: dict) -> None:
    hosts: dict[str, dict[str, dict]] = {}
    for triple, archive in manifest["archives"].items():
        host = HOSTS[triple]
        hosts.setdefault(host.buck_os, {})[host.buck_cpu] = {
            "sha256": archive["sha256"],
            "strip_prefix": archive["strip_prefix"],
            "url": archive["url"],
        }

    def starlark(value: object, indent: int) -> str:
        if not isinstance(value, dict):
            return json.dumps(value)
        inner = " " * (indent + 4)
        entries = "".join(
            f"{inner}{json.dumps(key)}: {starlark(value[key], indent + 4)},\n"
            for key in sorted(value)
        )
        return "{\n" + entries + " " * indent + "}"

    pin = {"hosts": hosts, "source_commit": manifest["source_commit"]}
    path.write_text(
        '"""Generated pin for the latest tested prebuilt PPC64 toolchain."""\n\n'
        'DEFAULT_PPC64_TOOLCHAIN = "prebuilt"\n\n'
        f"NIGHTLY_TOOLCHAIN = {starlark(pin, 0)}\n",
        encoding="utf-8",
    )


def release(args: argparse.Namespace) -> int:
    builds = [read_json(path) for path in sorted(args.hosts_dir.glob("xbox360-toolchain-*.json"))]
    if not builds:
        raise SystemExit(f"{args.hosts_dir} has no host toolchains")
    for field in ("source_commit", "toolchain_inputs"):
        values = {json.dumps(build[field], sort_keys=True) for build in builds}
        if len(values) != 1:
            raise SystemExit(f"the host toolchains disagree on {field}")

    first = builds[0]
    url = f"https://github.com/{args.repository}/releases/download/{args.release_tag}/"
    archives = {}
    for build in builds:
        archive = dict(build["archive"])
        archive["url"] = url + archive["name"]
        archive["llvm_tools"] = build["llvm_tools"]
        archive["rust_tools"] = build["rust_tools"]
        archives[build["host"]] = archive

    manifest = {
        **{name: first["toolchain_inputs"][name] for name in INPUT_FIELDS},
        "archives": archives,
        "schema": 2,
        "source_commit": first["source_commit"],
        "target_spec": first["target_spec"],
        "toolchain_inputs": first["toolchain_inputs"],
    }
    args.out_dir.mkdir(parents=True, exist_ok=True)
    write_json(args.out_dir / "xbox360-toolchain.json", manifest)
    write_buck_pin(args.out_dir / "nightly_toolchain.bzl", manifest)

    for triple, archive in archives.items():
        (args.out_dir / f"{archive['strip_prefix']}.buckconfig").write_text(
            "[xbox360]\n"
            "  ppc64_toolchain = prebuilt\n"
            f"  ppc64_toolchain_url = {archive['url']}\n"
            f"  ppc64_toolchain_sha256 = {archive['sha256']}\n"
            f"  ppc64_toolchain_strip_prefix = {archive['strip_prefix']}\n",
            encoding="utf-8",
        )

    # One descriptor per tool, for every host that has it.
    descriptors: dict[str, dict[str, dict]] = {}
    for triple, archive in archives.items():
        host = HOSTS[triple]
        for tool in archive["rust_tools"] + archive["llvm_tools"]:
            descriptors.setdefault(tool, {})[host.dotslash] = {
                "digest": archive["blake3"],
                "format": archive["format"],
                "hash": "blake3",
                "path": f"{archive['strip_prefix']}/bin/{tool}{host.executable_suffix}",
                "providers": [{"url": archive["url"]}],
                "size": archive["size"],
            }
    dotslash_dir = args.out_dir / "dotslash"
    dotslash_dir.mkdir(exist_ok=True)
    for tool, platforms in descriptors.items():
        write_dotslash(dotslash_dir / f"xbox360-{tool}", platforms)
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)

    package_parser = commands.add_parser("package", help="archive this host's toolchain")
    package_parser.add_argument("--host", choices=sorted(HOSTS), required=True)
    package_parser.add_argument("--rust-dist", type=Path, required=True)
    package_parser.add_argument("--llvm-dist", type=Path, required=True)
    package_parser.add_argument("--target-spec", type=Path, required=True)
    package_parser.add_argument("--out-dir", type=Path, required=True)
    package_parser.add_argument("--source-commit", required=True)
    package_parser.add_argument("--toolchain-inputs", type=Path, required=True)
    package_parser.add_argument("--source-date-epoch", type=int, required=True)
    package_parser.add_argument("--dotslash", type=Path, required=True)
    package_parser.add_argument("--runtime-library", type=Path, action="append", default=[])
    package_parser.add_argument("--compression-level", type=int, default=6)
    package_parser.set_defaults(run=package)

    release_parser = commands.add_parser("release", help="combine the host archives")
    release_parser.add_argument("--hosts-dir", type=Path, required=True)
    release_parser.add_argument("--out-dir", type=Path, required=True)
    release_parser.add_argument("--repository", required=True, help="GitHub owner/repository")
    release_parser.add_argument("--release-tag", required=True)
    release_parser.set_defaults(run=release)

    args = parser.parse_args()
    return args.run(args)


if __name__ == "__main__":
    raise SystemExit(main())
