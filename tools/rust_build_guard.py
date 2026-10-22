#!/usr/bin/env python3
"""Fail-closed free-space guard for Cargo commands."""

from __future__ import annotations

import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from typing import Callable, Mapping, Sequence


MIN_FREE_BYTES = 3_000_000_000_000
SUPPORTED_COMMANDS = {"build", "check", "clippy", "fmt", "doc", "run"}


class GuardError(RuntimeError):
    pass


def _exists(path: Path) -> bool:
    try:
        path.stat()
        return True
    except FileNotFoundError:
        return False
    except OSError as exc:
        raise GuardError(f"could not inspect path {path} ({type(exc).__name__})") from exc


def _effective_env(env: Mapping[str, str], cwd: Path, mise_command: str = "mise") -> dict[str, str]:
    """Resolve mise's project environment without exposing its values."""
    mise = mise_command if os.path.isabs(mise_command) else shutil.which(mise_command, path=env.get("PATH"))
    if not mise:
        raise GuardError("mise is unavailable; refusing to run Cargo")
    try:
        result = subprocess.run(
            [mise, "env", "--json"], cwd=cwd, env=dict(env),
            check=False, capture_output=True, text=True, timeout=15,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise GuardError(f"could not resolve mise environment ({type(exc).__name__})") from exc
    if result.returncode != 0:
        raise GuardError(f"mise env failed with status {result.returncode}")
    try:
        values = json.loads(result.stdout)
    except (ValueError, TypeError) as exc:
        raise GuardError("mise env returned invalid JSON") from exc
    if not isinstance(values, dict) or any(not isinstance(k, str) or not isinstance(v, str) for k, v in values.items()):
        raise GuardError("mise env returned an invalid environment map")
    merged = dict(env)
    merged.update(values)
    return merged


def _expand_path(value: str, env: Mapping[str, str], cwd: Path) -> Path:
    if not value:
        raise GuardError("empty path cannot be resolved")
    home = env.get("HOME") or env.get("USERPROFILE")
    expanded = value
    if expanded == "~" or expanded.startswith("~/") or expanded.startswith("~\\"):
        if not home:
            raise GuardError("cannot expand home-relative path")
        expanded = home + expanded[1:]

    def substitute(match: re.Match[str]) -> str:
        name = match.group(1) or match.group(2)
        if name not in env:
            raise GuardError(f"path refers to unset environment variable {name}")
        return env[name]

    expanded = re.sub(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}|\$([A-Za-z_][A-Za-z0-9_]*)", substitute, expanded)
    if "$" in expanded:
        raise GuardError("path contains an unsupported variable expression")
    path = Path(expanded)
    return path if path.is_absolute() else cwd / path


def _mbx_locations(env: Mapping[str, str], cwd: Path, home: Path) -> tuple[Path, Path]:
    """Return MBX cache and managed target roots using documented precedence."""
    system = sys.platform
    if system == "darwin":
        cache_default = home / "Library" / "Caches" / "mbx"
        config_path = home / "Library" / "Application Support" / "mbx" / "config.toml"
    elif os.name == "nt":
        local = env.get("LOCALAPPDATA")
        appdata = env.get("APPDATA")
        if not local or not appdata:
            raise GuardError("Windows MBX cache/config roots are unknown")
        cache_default = Path(local) / "mbx"
        config_path = Path(appdata) / "mbx" / "config.toml"
    else:
        cache_default = Path(env.get("XDG_CACHE_HOME", str(home / ".cache"))) / "mbx"
        config_path = Path(env.get("XDG_CONFIG_HOME", str(home / ".config"))) / "mbx" / "config.toml"

    config: dict = {}
    if _exists(config_path):
        try:
            parsed = tomllib.loads(config_path.read_text(encoding="utf-8"))
        except (OSError, UnicodeError, tomllib.TOMLDecodeError) as exc:
            raise GuardError("could not read MBX configuration") from exc
        if not isinstance(parsed, dict):
            raise GuardError("MBX configuration has an invalid shape")
        config = parsed
    configured_cache = config.get("cache_dir", str(cache_default))
    target_settings = config.get("target", {})
    if not isinstance(configured_cache, str) or not isinstance(target_settings, dict):
        raise GuardError("MBX cache configuration has an invalid shape")
    configured_target = target_settings.get("root")
    if configured_target is not None and not isinstance(configured_target, str):
        raise GuardError("MBX target root has an invalid shape")

    cache_value = env.get("MBX_CACHE_DIR", configured_cache)
    cache = _expand_path(cache_value, env, cwd)
    if not Path(cache_value).is_absolute() and not cache_value.startswith("~"):
        raise GuardError("relative MBX cache roots are not resolved by documented configuration")
    target_value = env.get("MBX_TARGET_ROOT", configured_target or str(cache / "targets"))
    target = _expand_path(target_value, env, cache)
    return cache, target


def discover_paths(env: Mapping[str, str], cwd: Path, temp_dir: str | None = None) -> list[Path]:
    home_value = env.get("HOME") or env.get("USERPROFILE")
    if not home_value:
        raise GuardError("home directory is unknown")
    home = _expand_path(home_value, env, cwd)
    paths = [cwd, home]
    cargo_home = env.get("CARGO_HOME", str(home / ".cargo"))
    paths.append(_expand_path(cargo_home, env, cwd))
    mbx_cache, mbx_targets = _mbx_locations(env, cwd, home)
    paths.extend((mbx_cache, mbx_targets))
    paths.append(_cargo_workspace_root(cwd) / "target")

    for name in ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR", "CARGO_BUILD_BUILD_DIR", "TMPDIR", "TMP", "TEMP"):
        value = env.get(name)
        if value:
            if name == "CARGO_BUILD_BUILD_DIR" and ("{" in value or "}" in value):
                raise GuardError("Cargo build-dir environment uses path templating the guard cannot resolve")
            paths.append(_expand_path(value, env, cwd))
    paths.extend(_cargo_config_output_paths(env, cwd, home))
    paths.append(Path(temp_dir) if temp_dir else Path(tempfile.gettempdir()))
    return paths


def _cargo_workspace_root(cwd: Path) -> Path:
    nearest_package: Path | None = None
    for directory in (cwd, *cwd.parents):
        manifest = directory / "Cargo.toml"
        if not _exists(manifest):
            continue
        try:
            content = tomllib.loads(manifest.read_text(encoding="utf-8"))
        except (OSError, UnicodeError, tomllib.TOMLDecodeError) as exc:
            raise GuardError("could not inspect Cargo workspace manifest") from exc
        if nearest_package is None and "package" in content:
            nearest_package = directory
        package = content.get("package", {})
        if isinstance(package, dict) and package.get("workspace"):
            raise GuardError("Cargo package has an external workspace root that cannot be resolved safely")
        if "workspace" in content:
            return directory
    return nearest_package or cwd


def _cargo_config_documents(env: Mapping[str, str], cwd: Path, home: Path) -> list[tuple[Path, dict]]:
    """Load applicable Cargo configs and reject indirect path-changing policy."""
    configs: list[Path] = []
    for directory in (cwd, *cwd.parents):
        for name in (".cargo/config.toml", ".cargo/config"):
            path = directory / name
            if _exists(path):
                configs.append(path)
    cargo_home = _expand_path(env.get("CARGO_HOME", str(home / ".cargo")), env, cwd)
    for name in ("config.toml", "config"):
        path = cargo_home / name
        if _exists(path):
            configs.append(path)
    documents: list[tuple[Path, dict]] = []
    for config_path in configs:
        try:
            config = tomllib.loads(config_path.read_text(encoding="utf-8"))
        except (OSError, UnicodeError, tomllib.TOMLDecodeError) as exc:
            raise GuardError("could not read Cargo configuration") from exc
        if "include" in config or "env" in config:
            raise GuardError("Cargo configuration include/environment can alter build outputs")
        documents.append((config_path, config))
    return documents


def _cargo_config_output_paths(env: Mapping[str, str], cwd: Path, home: Path) -> list[Path]:
    """Include configured Cargo output roots; inspect every applicable config."""
    paths: list[Path] = []
    for config_path, config in _cargo_config_documents(env, cwd, home):
        build = config.get("build", {})
        if not isinstance(build, dict):
            raise GuardError("Cargo build configuration has an invalid shape")
        for key in ("rustc-wrapper", "rustc-workspace-wrapper"):
            if key in build and build[key] is not None:
                raise GuardError(f"Cargo {key} output behavior is unknown")
        for key in ("target-dir", "build-dir"):
            value = build.get(key)
            if value is not None:
                if not isinstance(value, str):
                    raise GuardError(f"Cargo {key} must be a path string")
                if "{" in value or "}" in value:
                    raise GuardError(f"Cargo {key} uses path templating the guard cannot resolve")
                paths.append(_expand_path(value, env, config_path.parent.parent))
    return paths


def _has_nextest_alias(env: Mapping[str, str], cwd: Path, home: Path) -> bool:
    for _path, config in _cargo_config_documents(env, cwd, home):
        aliases = config.get("alias", {})
        if not isinstance(aliases, dict):
            raise GuardError("Cargo aliases have an invalid configuration shape")
        if "nextest" in aliases:
            return True
    return False


def filesystem_free_bytes(path: Path, *, stat_fn: Callable = os.stat, statvfs_fn: Callable = os.statvfs) -> tuple[int, int]:
    try:
        resolved = path.expanduser().resolve(strict=False)
        probe = resolved
        while not _exists(probe):
            parent = probe.parent
            if parent == probe:
                raise GuardError(f"no existing filesystem ancestor for {path}")
            probe = parent
        st = stat_fn(probe)
        stats = statvfs_fn(probe)
        bavail, frsize = stats.f_bavail, stats.f_frsize
    except GuardError:
        raise
    except (OSError, RuntimeError, ValueError) as exc:
        raise GuardError(f"filesystem check failed for {path} ({type(exc).__name__})") from exc
    if not isinstance(bavail, int) or not isinstance(frsize, int) or bavail < 0 or frsize <= 0:
        raise GuardError(f"filesystem returned invalid free-space values for {path}")
    free = bavail * frsize
    if free > sys.maxsize:
        raise GuardError(f"filesystem free-space value overflows for {path}")
    return st.st_dev, free


def check_space(paths: Sequence[Path], *, stat_fn: Callable = os.stat, statvfs_fn: Callable = os.statvfs, minimum: int = MIN_FREE_BYTES) -> dict[int, int]:
    if minimum != MIN_FREE_BYTES:
        raise GuardError("the required free-space threshold is fixed")
    devices: dict[int, int] = {}
    for path in paths:
        device, free = filesystem_free_bytes(path, stat_fn=stat_fn, statvfs_fn=statvfs_fn)
        devices[device] = min(devices.get(device, free), free)
    if not devices:
        raise GuardError("no filesystems were checked")
    short = {device: free for device, free in devices.items() if free < MIN_FREE_BYTES}
    if short:
        detail = ", ".join(f"device {dev}: {free} bytes" for dev, free in sorted(short.items()))
        raise GuardError(f"free space is below {MIN_FREE_BYTES} bytes ({detail})")
    return devices


def main(argv: Sequence[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    if args and args[0] == "--":
        args.pop(0)
    if len(args) < 4 or args[0] != "mise" or args[1] != "exec" or args[2] != "--" or args[3] != "cargo":
        print("usage: rust_build_guard.py -- mise exec -- cargo [ARG ...]", file=sys.stderr)
        return 2
    cargo_args = args[4:]
    if not cargo_args:
        print("rust build guard: denied: Cargo subcommand is required", file=sys.stderr)
        return 125
    subcommand = cargo_args[0]
    if subcommand not in SUPPORTED_COMMANDS and subcommand != "nextest":
        print(f"rust build guard: denied: unsupported Cargo command {subcommand!r}", file=sys.stderr)
        return 125
    if subcommand == "nextest" and (len(cargo_args) < 2 or cargo_args[1] not in {"run", "list"}):
        print("rust build guard: denied: only cargo nextest run/list are supported", file=sys.stderr)
        return 125
    if any(
        arg == "--config" or arg.startswith("--config=")
        or arg == "--manifest-path" or arg.startswith("--manifest-path=")
        for arg in cargo_args
    ):
        print("rust build guard: denied: Cargo --config and --manifest-path overrides are not supported", file=sys.stderr)
        return 125
    cwd = Path.cwd()
    try:
        effective = _effective_env(os.environ, cwd, args[0])
        if effective.get("RUSTC_WRAPPER") or effective.get("RUSTC_WORKSPACE_WRAPPER"):
            raise GuardError("custom rustc wrapper output behavior is unknown")
        if effective.get("CARGO_BUILD_RUSTC_WRAPPER") or effective.get("CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"):
            raise GuardError("Cargo rustc wrapper environment output behavior is unknown")
        home_value = effective.get("HOME") or effective.get("USERPROFILE")
        if not home_value:
            raise GuardError("home directory is unknown")
        home = _expand_path(home_value, effective, cwd)
        if subcommand == "nextest" and _has_nextest_alias(effective, cwd, home):
            raise GuardError("Cargo nextest is configured as an alias; refusing ambiguous command")
        paths = discover_paths(effective, cwd)
        for index, arg in enumerate(cargo_args):
            if arg == "--target-dir":
                if index + 1 >= len(cargo_args):
                    raise GuardError("Cargo --target-dir has no path")
                paths.append(_expand_path(cargo_args[index + 1], effective, cwd))
            elif arg.startswith("--target-dir="):
                paths.append(_expand_path(arg.split("=", 1)[1], effective, cwd))
        checked = check_space(paths)
    except GuardError as exc:
        print(f"rust build guard: denied: {exc}", file=sys.stderr)
        return 125
    print(f"rust build guard: allowed; checked {len(checked)} filesystem(s)", file=sys.stderr)
    os.execvpe(args[0], args, os.environ.copy())
    return 125


if __name__ == "__main__":
    raise SystemExit(main())
