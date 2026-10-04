"""Static call-order contracts; these are not native Windows security evidence."""

from pathlib import Path
import re
import unittest
import xml.etree.ElementTree as ET


WINDOWS = Path(__file__).resolve().parents[1]


def function(source: str, name: str) -> str:
    start = re.search(rf"^function {re.escape(name)} \{{\n", source, re.MULTILINE)
    if start is None:
        raise AssertionError("missing function")
    next_function = re.search(r"^(?:function \S+ \{\n|# Only one elevated installer)", source[start.end():], re.MULTILINE)
    if next_function is None:
        raise AssertionError("missing following source boundary")
    return source[start.end():start.end() + next_function.start()]


def ordered(source: str, *tokens: str) -> None:
    offsets = [source.index(token) for token in tokens]
    if offsets != sorted(offsets) or len(set(offsets)) != len(offsets):
        raise AssertionError("incorrect admission order")


class StateAdmissionOrder(unittest.TestCase):
    def test_verified_candidate_dominates_install_update_and_recovery(self) -> None:
        source = (WINDOWS / "Install-DeviceHost.ps1").read_text(encoding="utf-8")
        bundle = function(source, "Read-SignedBundle")
        ordered(bundle,
                "Assert-SignedBy -LiteralPath $hostPath",
                "Assert-Sha256 -LiteralPath $hostPath",
                "return [pscustomobject]")
        main = source[source.index("# Only one elevated installer"):]
        install_start = main.index("if ($Action -eq 'Install')")
        update_start = main.index("} elseif ($Action -eq 'Update')")
        install = main[install_start:update_start]
        ordered(install, "$bundle = Read-SignedBundle", "Assert-StateAdmission -Bundle $bundle",
                "New-Item -ItemType Directory -Path $installParent")
        ordered(main[update_start:], "$bundle = Read-SignedBundle", "Assert-StateAdmission -Bundle $bundle",
                "$pendingUpdate = Read-PendingUpdate", "Complete-PendingUpdate",
                "Recover-UnjournaledUpdate", "Bundle $($bundle.Version) is already installed")
        self.assertEqual(install.count("Assert-StateAdmission -Bundle $bundle"), 1)
        self.assertEqual(main[update_start:].count("Assert-StateAdmission -Bundle $bundle"), 1)
        self.assertNotIn("Assert-StateAdmission", function(source, "Complete-PendingUpdate"))
        self.assertNotIn("Assert-StateAdmission", function(source, "Recover-UnjournaledUpdate"))
        # Existing mutation helpers remain below the one verified call in their only main entry.
        self.assertLess(main[update_start:].index("Assert-StateAdmission"),
                        main[update_start:].index("Write-PendingUpdate"))

    def test_child_is_exact_bounded_and_joined_without_output_disclosure(self) -> None:
        source = (WINDOWS / "Install-DeviceHost.ps1").read_text(encoding="utf-8")
        body = function(source, "Assert-StateAdmission")
        ordered(body, "$payload = [IO.File]::Open", "SHA256]::HashData($payload)",
                "Assert-SignedBy", "$process.Start()")
        self.assertIn("[IO.FileShare]::Read", body)
        self.assertIn("$start.Environment.Clear()", body)
        self.assertIn("$start.UseShellExecute = $false", body)
        self.assertEqual(body.count("$start.ArgumentList.Add('prepare-state')"), 1)
        self.assertEqual(body.count("$process.Start()"), 1)
        self.assertIn("$clock.ElapsedMilliseconds -ge 15000", body)
        self.assertIn("$stdout.Length + $count -gt 1024", body)
        self.assertIn("$stderr.Length + $count -gt 1024", body)
        self.assertIn("$process.WaitForExit(0)", body)
        self.assertIn("$process.Kill($true)", body)
        self.assertIn("$process.WaitForExit($remaining)", body)
        self.assertIn("$cleanup.ElapsedMilliseconds -ge 5000", body)
        self.assertIn("if (-not $task.IsCompleted)", body)
        self.assertIn("$stdout.ToString() -cne", body)
        self.assertNotIn("ReadToEnd", body)
        self.assertNotRegex(body, r"Write-(Output|Warning|Error|Host).*\$(stdout|stderr)")

    def test_preparation_has_no_http_or_content_read_and_test_hooks_are_excluded(self) -> None:
        program = (WINDOWS / "RiAuth.DeviceHost" / "Program.cs").read_text(encoding="utf-8")
        ordered(program, 'args[0] == "prepare-state"', "new WindowsStateStore().Prepare()",
                'Console.Out.Write("RIAUTH-STATE-READY-V1', "using var http = new HttpClient")
        self.assertIn('if (args.Length != 1)', program)
        store = (WINDOWS / "RiAuth.DeviceHost" / "WindowsStateStore.cs").read_text(encoding="utf-8")
        production = re.sub(r"(?ms)^#if DEVICE_STATE_TESTS\n.*?^#endif\n", "", store)
        for token in ("SetNamedSecurityInfo", "TestContentReads", "TestDecrypts", "fixtureRoot",
                      "TestCreateAnchor", "TestSetMetadata", "TestSecurityBytes"):
            self.assertNotIn(token, production)
        prepare = production[production.index("internal void Prepare()"):production.index("public DeviceState? Load()")]
        for token in ("ReadState", "Unprotect", "HttpClient", "Deserialize"):
            self.assertNotIn(token, prepare)
        ordered(production[production.index("public DeviceState? Load()"):],
                "directory.OpenState", "WinSecurity.ReadState(stream)", "WinSecurity.Unprotect")
        self.assertNotIn("File.ReadAllBytes", production)
        self.assertNotIn("File.Replace", production)
        self.assertNotIn("File.Delete", production)
        self.assertIn("if (!published) MarkDeleted(temporary)", production)
        self.assertIn("parent.DangerousAddRef", production)
        self.assertIn("ShareRead | ShareWrite | ShareDelete", production)

    def test_native_target_links_actual_source_without_publish_or_packages(self) -> None:
        tree = ET.fromstring((WINDOWS / "RiAuth.DeviceHost.StateTests" /
                              "RiAuth.DeviceHost.StateTests.csproj").read_text(encoding="utf-8"))
        self.assertEqual(tree.findtext("PropertyGroup/TargetFramework"), "net9.0")
        self.assertEqual(tree.findtext("PropertyGroup/IsPublishable"), "false")
        self.assertEqual(tree.findtext("PropertyGroup/DefineConstants"),
                         "$(DefineConstants);DEVICE_STATE_TESTS")
        self.assertEqual([node.attrib["Include"] for node in tree.findall("ItemGroup/Compile")],
                         ["../RiAuth.DeviceHost/WindowsStateStore.cs", "../RiAuth.DeviceHost/DeviceHost.cs",
                          "../RiAuth.DeviceHost/WindowsLocalAccount.cs"])
        self.assertEqual(tree.findall(".//PackageReference"), [])
        production_project = (WINDOWS / "RiAuth.DeviceHost" / "RiAuth.DeviceHost.csproj").read_text()
        self.assertNotIn("DEVICE_STATE_TESTS", production_project)


if __name__ == "__main__":
    unittest.main()
