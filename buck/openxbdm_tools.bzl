# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

_RELEASE = "nightly-57c60d10a769"
_RELEASE_URL = "https://github.com/landaire/openxbdm/releases/download/{}/".format(_RELEASE)

_HOSTS = {
    "linux-x86_64": struct(
        archive = "xeedee-nightly-x86_64-unknown-linux-gnu.tar.gz",
        sha256 = "4c033ec015fa5c48bcb1384ee88f32fea559c3b0beca4f577f2647f68853e8a7",
        strip_prefix = "xeedee-nightly-x86_64-unknown-linux-gnu",
        suffix = "",
    ),
    "macos-aarch64": struct(
        archive = "xeedee-nightly-aarch64-apple-darwin.tar.gz",
        sha256 = "553500680eb736138eba04127e9bdb65552d1a2ba6f13411bb0f5204863fe30d",
        strip_prefix = "xeedee-nightly-aarch64-apple-darwin",
        suffix = "",
    ),
    "windows-x86_64": struct(
        archive = "xeedee-nightly-x86_64-pc-windows-msvc.zip",
        sha256 = "a1607fb19f661e66b0f6ed1a054fba3714c65ec176652de75ba76d47b5466fcf",
        strip_prefix = "xeedee-nightly-x86_64-pc-windows-msvc",
        suffix = ".exe",
    ),
}


def _host_value(values, unsupported):
    return select({
        "prelude//os:linux": select({
            "prelude//cpu:x86_64": values["linux-x86_64"],
            "DEFAULT": unsupported,
        }),
        "prelude//os:macos": select({
            "prelude//cpu:arm64": values["macos-aarch64"],
            "DEFAULT": unsupported,
        }),
        "prelude//os:windows": select({
            "prelude//cpu:x86_64": values["windows-x86_64"],
            "DEFAULT": unsupported,
        }),
        "DEFAULT": unsupported,
    })


def _archive_tool_impl(ctx):
    if ctx.attrs.archive == None:
        fail("downloaded openxbdm tools support Linux x86_64, macOS aarch64, and Windows x86_64; configure xbox360.xbdm_tool and xbox360.kd_tool on this host")
    return [
        DefaultInfo(),
        RunInfo(args = cmd_args(ctx.attrs.archive, format = "{}/" + ctx.attrs.path)),
    ]


_archive_tool = rule(
    impl = _archive_tool_impl,
    attrs = {
        "archive": attrs.option(attrs.source(), default = None),
        "path": attrs.string(),
    },
)


def openxbdm_host_tools():
    for host, release in _HOSTS.items():
        native.http_archive(
            name = "openxbdm-tools-" + host,
            sha256 = release.sha256,
            strip_prefix = release.strip_prefix,
            urls = [_RELEASE_URL + release.archive],
        )

    archives = {host: ":openxbdm-tools-" + host for host in _HOSTS}
    for tool in ["xeedee", "openxbdm-kd"]:
        paths = {host: tool + release.suffix for host, release in _HOSTS.items()}
        _archive_tool(
            name = tool,
            archive = _host_value(archives, None),
            path = _host_value(paths, ""),
            visibility = ["PUBLIC"],
        )
