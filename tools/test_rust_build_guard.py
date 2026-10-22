from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

import rust_build_guard as guard


class Fs:
    def __init__(self, free_by_device: dict[int, int]):
        self.free_by_device = free_by_device

    def stat(self, path: Path):
        return type("Stat", (), {"st_dev": int(str(path).split("device-")[-1]) if "device-" in str(path) else 1})()

    def statvfs(self, path: Path):
        device = self.stat(path).st_dev
        free = self.free_by_device[device]
        return type("Vfs", (), {"f_bavail": free, "f_frsize": 1})()


class BuildGuardTests(unittest.TestCase):
    def test_rejects_one_byte_below_and_accepts_threshold(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "device-1"
            path.mkdir()
            fs = Fs({1: guard.MIN_FREE_BYTES - 1})
            with self.assertRaisesRegex(guard.GuardError, "below"):
                guard.check_space([path], stat_fn=fs.stat, statvfs_fn=fs.statvfs)
            fs.free_by_device[1] = guard.MIN_FREE_BYTES
            self.assertEqual(guard.check_space([path], stat_fn=fs.stat, statvfs_fn=fs.statvfs), {1: guard.MIN_FREE_BYTES})

    def test_checks_every_distinct_device(self):
        with tempfile.TemporaryDirectory() as tmp:
            paths = [Path(tmp) / f"device-{n}" for n in (1, 2)]
            for path in paths:
                path.mkdir()
            fs = Fs({1: guard.MIN_FREE_BYTES, 2: guard.MIN_FREE_BYTES - 1})
            with self.assertRaisesRegex(guard.GuardError, "device 2"):
                guard.check_space(paths, stat_fn=fs.stat, statvfs_fn=fs.statvfs)

    def test_symlink_and_nonexistent_leaf_resolve_to_target_device(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            actual = root / "real"
            actual.mkdir()
            link = root / "link"
            link.symlink_to(actual, target_is_directory=True)
            missing = link / "future" / "target"
            device, free = guard.filesystem_free_bytes(missing)
            self.assertEqual(device, os.stat(actual).st_dev)
            self.assertGreaterEqual(free, 0)

    def test_stat_failure_denies(self):
        def fail(_path):
            raise OSError("injected")
        with self.assertRaisesRegex(guard.GuardError, "filesystem check failed"):
            guard.filesystem_free_bytes(Path("/"), stat_fn=fail)

    def test_invalid_or_overflowing_statvfs_denies(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)
            st = lambda _path: type("Stat", (), {"st_dev": 1})()
            for bavail, frsize in ((0, 0), (-1, 1), (sys.maxsize, 2)):
                vfs = lambda _path, bavail=bavail, frsize=frsize: type("Vfs", (), {"f_bavail": bavail, "f_frsize": frsize})()
                with self.assertRaises(guard.GuardError):
                    guard.filesystem_free_bytes(path, stat_fn=st, statvfs_fn=vfs)

    def test_threshold_environment_variable_is_ignored(self):
        fs = Fs({1: guard.MIN_FREE_BYTES - 1})
        with mock.patch.dict(os.environ, {"RUST_BUILD_GUARD_MIN_FREE_BYTES": "1"}):
            with self.assertRaises(guard.GuardError):
                guard.check_space([Path("/device-1")], stat_fn=fs.stat, statvfs_fn=fs.statvfs)

    def test_command_argv_and_environment_are_preserved_on_exec(self):
        original = {"PATH": "/tools", "TOKEN_FOR_TEST": "unchanged"}
        args = ["mise", "exec", "--", "cargo", "nextest", "run", "two words"]
        with mock.patch.object(guard, "_effective_env", return_value={**original, "MISE_ONLY": "for-discovery"}), \
                mock.patch.object(guard, "discover_paths", return_value=[Path("/")]), \
                mock.patch.object(guard, "check_space", return_value={1: guard.MIN_FREE_BYTES}), \
                mock.patch.object(guard.os, "execvpe", side_effect=OSError("stop")) as execvpe, \
                mock.patch.dict(os.environ, original, clear=True):
            with self.assertRaises(OSError):
                guard.main(args)
            execvpe.assert_called_once_with(args[0], args, original)

    def test_cargo_config_roots_are_included(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / ".cargo").mkdir()
            (root / ".cargo/config.toml").write_text('[build]\ntarget-dir="../target-output"\nbuild-dir="../intermediate"\n')
            paths = guard._cargo_config_output_paths({"HOME": str(root)}, root, root)
            self.assertGreaterEqual(len(paths), 2)
            self.assertTrue({"target-output", "intermediate"}.issubset({p.name for p in paths}))

    def test_relative_mbx_cache_root_fails_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaisesRegex(guard.GuardError, "relative MBX cache"):
                guard._mbx_locations({"HOME": tmp, "MBX_CACHE_DIR": "relative/cache"}, Path(tmp), Path(tmp))

    def test_absolute_cargo_path_is_rejected_to_preserve_mbx_shim(self):
        with mock.patch.object(guard, "_effective_env") as mise_env:
            self.assertEqual(guard.main(["mise", "exec", "--", "/usr/bin/cargo", "build"]), 2)
            mise_env.assert_not_called()

    def test_cargo_configured_rustc_wrappers_fail_closed(self):
        for key in ("rustc-wrapper", "rustc-workspace-wrapper"):
            with self.subTest(key=key), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                (root / ".cargo").mkdir()
                (root / ".cargo/config.toml").write_text(f'[build]\n{key}="/custom/wrapper"\n')
                with self.assertRaisesRegex(guard.GuardError, "wrapper"):
                    guard._cargo_config_output_paths({"HOME": tmp}, root, root)

    def test_environment_build_dir_template_fails_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaisesRegex(guard.GuardError, "templating"):
                guard.discover_paths({"HOME": tmp, "CARGO_BUILD_BUILD_DIR": "{workspace-root}/build"}, Path(tmp), tmp)

    def test_real_deny_probe_does_not_launch_child(self):
        # On constrained disks this proves the guard exits before any command.
        # If the disk is above policy, the test is skipped to avoid running a
        # child under conditions where no denial is expected.
        _, free = guard.filesystem_free_bytes(Path.cwd())
        if free >= guard.MIN_FREE_BYTES:
            self.skipTest("filesystem is above the hardcoded threshold")
        script = Path(guard.__file__).resolve()
        result = subprocess.run(
            [sys.executable, str(script), "--", "mise", "exec", "--", "cargo", "--version"],
            cwd=Path.cwd(), capture_output=True, text=True, timeout=20,
            env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1", "RUST_BUILD_GUARD_MIN_FREE_BYTES": "1"},
        )
        self.assertEqual(result.returncode, 125, result.stderr)
        self.assertIn("denied", result.stderr)

    def test_cargo_output_config_override_fails_closed(self):
        with mock.patch.object(guard, "_effective_env") as mise_env:
            self.assertEqual(guard.main(["mise", "exec", "--", "cargo", "--config", "build.target-dir=/tmp/out"]), 125)
            mise_env.assert_not_called()


if __name__ == "__main__":
    unittest.main()
