"""Source contract for the Windows manifest signature/parse boundary.

Authenticode validation needs Windows, so this check guards the call path that
previously verified a mutable path and then reopened it for parsing.
"""

from pathlib import Path
import re
import unittest


INSTALLER = Path(__file__).resolve().parents[1] / "Install-DeviceHost.ps1"


def function_body(source: str, name: str) -> str:
    start = re.search(rf"^function {re.escape(name)} \{{\n", source, re.MULTILINE)
    if start is None:
        raise AssertionError(f"missing {name}")
    next_function = re.search(r"^function \S+ \{\n", source[start.end() :], re.MULTILINE)
    if next_function is None:
        raise AssertionError(f"missing function after {name}")
    return source[start.end() : start.end() + next_function.start()]


class ManifestBindingContract(unittest.TestCase):
    def test_signature_and_parser_consume_one_byte_array(self) -> None:
        source = INSTALLER.read_text(encoding="utf-8")
        verifier = function_body(source, "Assert-SignedScriptBytesBy")
        reader = function_body(source, "Read-SignedBundle")

        self.assertIn("$PSVersionTable.PSVersion.Minor -lt 4", verifier)
        self.assertIn(
            "Get-AuthenticodeSignature -Content $Content -SourcePathOrExtension 'ps1'",
            verifier,
        )
        self.assertIn("$signature.Status -ne 'Valid'", verifier)
        self.assertIn("$actual -cne $ExpectedThumbprint", verifier)

        open_source = "$manifestStream = [IO.File]::Open("
        cap = "if ($manifestStream.Length -gt 1048576)"
        read = "$manifestStream.ReadExactly($manifestBytes, 0, $manifestBytes.Length)"
        verify = "Assert-SignedScriptBytesBy -Content $manifestBytes"
        decode = "$manifestText = $utf8.GetString($manifestBytes)"
        first_line = "$firstLine = $manifestText.Substring(0, $lineEnd)"
        offsets = [reader.index(part) for part in (open_source, cap, read, verify, decode, first_line)]
        self.assertEqual(offsets, sorted(offsets))
        self.assertIn("[IO.FileShare]::Read", reader[offsets[0] : offsets[2]])
        self.assertIn("$manifestBytes = [byte[]]::new([int]$manifestStream.Length)", reader)
        self.assertEqual(reader.count("[IO.File]::Open("), 1)
        self.assertNotIn("Get-Content -LiteralPath $manifestPath", reader)
        self.assertNotIn("Assert-SignedBy -LiteralPath $manifestPath", reader)
        self.assertNotRegex(reader[offsets[3] : offsets[4]], r"\$manifestBytes\s*=")


if __name__ == "__main__":
    unittest.main()
