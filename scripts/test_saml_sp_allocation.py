#!/usr/bin/env python3
"""Check only the shell helper's temporary-output allocation and cleanup."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parent / "test-saml-sp.sh"
START = 'if [[ -n "${RIAUTH_LASSO_SP_BIN:-}" ]]; then\n'
END = "# Word splitting is the pkg-config argument list.\n"
OBSERVE = """import json, os, stat, sys
path = sys.argv[1]
info = os.stat(os.path.dirname(path))
print(json.dumps({"output": path, "mode": stat.S_IMODE(info.st_mode),
                  "uid": info.st_uid}))
"""


class AllocationTests(unittest.TestCase):
    def allocate(self, root, override=None, exit_code=0):
        source = SCRIPT.read_text()
        self.assertEqual(source.count(START), 1)
        self.assertEqual(source.count(END), 1)
        start = source.index(START)
        end = source.index(END, start)
        allocation = source[start:end]
        # The compiler, package probes, generated helper and Cargo are excluded.
        self.assertNotIn("riauth_cc", allocation)
        self.assertNotIn("riauth_cargo", allocation)
        shell = (
            "set -euo pipefail\n"
            + allocation
            + '\n"$1" -c "$2" "$riauth_out"\nexit "$3"\n'
        )
        env = {"PATH": os.defpath, "TMPDIR": str(root)}
        if override is not None:
            env["RIAUTH_LASSO_SP_BIN"] = str(override)
        result = subprocess.run(
            [shutil.which("bash"), "-c", shell, "allocation", sys.executable,
             OBSERVE, str(exit_code)],
            env=env, text=True, capture_output=True, timeout=5,
        )
        self.assertEqual(result.returncode, exit_code)
        self.assertEqual(result.stderr, "")
        observed = json.loads(result.stdout)
        self.assertEqual(set(observed), {"output", "mode", "uid"})
        return observed

    def test_default_outputs_are_unique_private_and_removed(self):
        with tempfile.TemporaryDirectory(prefix="saml-allocation-test-") as temporary:
            root = Path(temporary)
            neighbor = root / "riauth-i04-saml-sp"
            neighbor.mkdir()
            marker = neighbor / "lasso-saml-sp"
            marker.write_bytes(b"preserve")
            outputs = []
            for override in (None, ""):
                observed = self.allocate(root, override=override)
                output = Path(observed["output"])
                self.assertEqual(output.name, "lasso-saml-sp")
                self.assertTrue(output.parent.parent == root)
                self.assertEqual(observed["mode"], 0o700)
                self.assertEqual(observed["uid"], os.getuid())
                self.assertFalse(output.parent.exists())
                outputs.append(output)
            self.assertTrue(outputs[0] != outputs[1])
            self.assertEqual(marker.read_bytes(), b"preserve")

    def test_default_output_is_removed_after_failure(self):
        with tempfile.TemporaryDirectory(prefix="saml-allocation-test-") as temporary:
            observed = self.allocate(Path(temporary), exit_code=7)
            self.assertFalse(Path(observed["output"]).parent.exists())

    def test_explicit_output_and_parent_remain_caller_owned(self):
        with tempfile.TemporaryDirectory(prefix="saml-allocation-test-") as temporary:
            root = Path(temporary)
            output = root / "explicit" / "helper"
            observed = self.allocate(root, override=output)
            self.assertTrue(observed["output"] == str(output))
            self.assertTrue(output.parent.is_dir())
            marker = output.parent / "marker"
            marker.write_bytes(b"preserve")
            self.allocate(root, override=output, exit_code=7)
            self.assertEqual(marker.read_bytes(), b"preserve")


if __name__ == "__main__":
    unittest.main()
