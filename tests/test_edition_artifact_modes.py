"""Filesystem mode checks with all image/process operations stubbed."""

import ast
import pathlib
import stat
import tempfile
import types
import unittest


SCRIPT = pathlib.Path(__file__).resolve().parent.parent / "scripts/check-edition-artifacts.py"


class ContainerPreflightModes(unittest.TestCase):
    def test_only_mounted_executable_changes_mode_at_preflight(self):
        tree = ast.parse(SCRIPT.read_bytes(), filename=str(SCRIPT))
        selected = [node for node in tree.body
                    if isinstance(node, ast.FunctionDef) and node.name == "check_images"]
        self.assertEqual(len(selected), 1)
        with tempfile.TemporaryDirectory(prefix="edition-mode-test-") as temporary:
            parent = pathlib.Path(temporary) / "maintenance"
            parent.mkdir(mode=0o700)
            binary = parent / "riauth-maintenance"
            other = parent / "riauthctl"
            config = parent / "riauth.toml"
            key = parent / "database.key"
            for path, mode in [(binary, 0o700), (other, 0o700),
                               (config, 0o600), (key, 0o600)]:
                path.write_bytes(b"synthetic fixture")
                path.chmod(mode)

            def check_modes(executable_mode):
                for path, expected in [(binary, executable_mode), (other, 0o700),
                                       (parent, 0o700), (config, 0o600), (key, 0o600)]:
                    self.assertEqual(stat.S_IMODE(path.stat().st_mode), expected)

            images = []
            calls = []
            decisions = []

            def docker(*args):
                return args[-1] if args[:2] == ("image", "inspect") else ""

            def check_image(edition, *_args):
                check_modes(0o700)
                images.append(edition)

            def run(args, **_kwargs):
                preflight = "--entrypoint" in args
                check_modes(0o755 if preflight else 0o700)
                self.assertNotIn("--user", args)
                if preflight:
                    self.assertIn(f"{binary.resolve()}:/usr/local/bin/riauth-maintenance:ro", args)
                result = "preflight" if preflight else "downgrade"
                calls.append(result)
                return result

            namespace = {
                "pathlib": pathlib, "docker": docker, "port": lambda: 9000,
                "check_image": check_image, "subprocess": types.SimpleNamespace(run=run),
                "assert_downgrade_rejection": lambda *args: decisions.append(args),
                "print": lambda *_args: None,
            }
            # Only this reviewed function is evaluated, never the script's imports/main.
            module = ast.Module(body=selected, type_ignores=[])
            exec(compile(module, str(SCRIPT), "exec"), namespace)
            namespace["check_images"]("essentials", "platform", binary)
            check_modes(0o755)
            self.assertEqual(images, ["essentials", "platform"])
            self.assertEqual(calls, ["downgrade", "preflight"])
            self.assertEqual(decisions, [("downgrade", "preflight")])


if __name__ == "__main__":
    unittest.main()
