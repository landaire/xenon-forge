"""Pin for the latest tested prebuilt PPC64 toolchain."""

DEFAULT_PPC64_TOOLCHAIN = "prebuilt"

NIGHTLY_TOOLCHAIN = {
    "hosts": {
        "linux": {
            "x86_64": {
                "sha256": "720535e797486ab72dc208b65d2cc019b88fbd9d13b176f6d871481cbd73c0ae",
                "strip_prefix": "xbox360-toolchain-x86_64-unknown-linux-gnu",
                "url": "https://github.com/landaire/xenon-forge/releases/download/toolchain-nightly-c9997c1d885d/xbox360-toolchain-x86_64-unknown-linux-gnu.tar.xz",
            },
        },
        "macos": {
            "arm64": {
                "sha256": "e037b503e12da3d122b66e4364d609cee9b25cf8993415ac6f7c1eacaa049b8e",
                "strip_prefix": "xbox360-toolchain-aarch64-apple-darwin",
                "url": "https://github.com/landaire/xenon-forge/releases/download/toolchain-nightly-c9997c1d885d/xbox360-toolchain-aarch64-apple-darwin.tar.xz",
            },
        },
        "windows": {
            "x86_64": {
                "sha256": "777bad411ca3e82fdeb69ae6813da105f829df6c3e72c2254ea081acdcd9ed51",
                "strip_prefix": "xbox360-toolchain-x86_64-pc-windows-msvc",
                "url": "https://github.com/landaire/xenon-forge/releases/download/toolchain-nightly-c9997c1d885d/xbox360-toolchain-x86_64-pc-windows-msvc.tar.gz",
            },
        },
    },
    "source_commit": "a0e1737130259f114fd176cea02984cafa1ed974",
}
