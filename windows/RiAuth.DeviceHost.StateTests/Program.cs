using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Runtime.ExceptionServices;
using System.Runtime.Versioning;
using System.Security.Cryptography;
using System.Security.Principal;
using System.Text.Json;
using RiAuth.DeviceHost;

namespace RiAuth.DeviceHost.StateTests;

internal static class Program
{
    private static string currentCase = "preflight";
    private static readonly List<string> passed = [];
    private static bool cleanupFailed;
    private static readonly Stopwatch clock = Stopwatch.StartNew();
    private const string DirectorySddl = "O:BAG:BAD:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)";
    private const string FileSddl = "O:BAG:BAD:P(A;;FA;;;SY)(A;;FA;;;BA)";

    [DllImport("kernel32.dll", EntryPoint = "CreateHardLinkW", ExactSpelling = true,
        CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern int CreateHardLink(string name, string existing, IntPtr attributes);

    private static int Main(string[] args)
    {
        if (args.Length != 0 || !OperatingSystem.IsWindows() || IntPtr.Size != 8
            || RuntimeInformation.ProcessArchitecture != Architecture.X64)
        {
            Console.WriteLine("{\"status\":\"unsupported_profile\"}");
            return 77;
        }
        // This is an owned test-process bound, not a synchronous kernel-I/O quota.
        using var watchdog = new Timer(_ => Environment.Exit(124), null, 100000, Timeout.Infinite);
        try
        {
            RunWindows();
            Require(clock.Elapsed < TimeSpan.FromSeconds(90), "test deadline");
            Console.WriteLine(JsonSerializer.Serialize(new
            {
                status = "passed", cases = passed, elapsed_ms = clock.ElapsedMilliseconds,
                ordinary_token_namespace = "not_run_requires_standard_token_fixture",
                physical_credential_provider = "not_run", cleanup = "owned_fixture_paths_removed"
            }));
            return 0;
        }
        catch (Exception error)
        {
            Console.WriteLine(JsonSerializer.Serialize(new
            {
                status = "failed", case_name = currentCase, passed,
                native_code = error is Win32Exception native ? (int?)native.NativeErrorCode : null,
                elapsed_ms = clock.ElapsedMilliseconds, cleanup_failed = cleanupFailed
            }));
            return 1;
        }
    }

    [SupportedOSPlatform("windows")]
    private static void RunWindows()
    {
        using var identity = WindowsIdentity.GetCurrent();
        Require(new WindowsPrincipal(identity).IsInRole(WindowsBuiltInRole.Administrator), "elevation");
        var user = identity.User?.Value ?? throw new InvalidOperationException("missing test actor");
        Require(user is not ("S-1-5-18" or "S-1-5-32-544"), "separate test actor");
        Case("native_layouts", WinSecurity.TestLayouts);
        Case("create_load_save_purge", Lifecycle);
        foreach (var place in new[] { "RiAuth", "DeviceHost", "device.json" })
        {
            var isDirectory = place != "device.json";
            var flags = isDirectory ? "OICI" : "";
            Case("original_owner_" + place, () => Refusal(place,
                $"O:{user}G:BAD:P(A;{flags};FA;;;SY)(A;{flags};FA;;;BA)"));
            Case("additional_write_" + place, () => Refusal(place,
                $"O:BAG:BAD:P(A;{flags};FA;;;SY)(A;{flags};FA;;;BA)(A;;FW;;;BU)"));
            Case("unprotected_" + place, () => Refusal(place,
                $"O:BAG:BAD:(A;{flags};FA;;;SY)(A;{flags};FA;;;BA)"));
            Case("duplicate_trustee_" + place, () => Refusal(place,
                $"O:BAG:BAD:P(A;{flags};FA;;;SY)(A;{flags};FA;;;SY)"));
            Case("null_dacl_" + place, () => Refusal(place, "O:BAG:BAD:NO_ACCESS_CONTROL"));
        }
        // Assigning SY ownership requires the native test actor's owner-assignment
        // privilege. Absence is a real refusal/failure, never a success or repair.
        Case("system_owner_preserved", () =>
        {
            using var fixture = new Fixture();
            WinSecurity.TestSetMetadata(fixture.StatePath, "O:SYG:BAD:P(A;;FA;;;SY)(A;;FA;;;BA)");
            var before = fixture.Snapshot();
            Require(fixture.Store.Load() == fixture.State, "system owner load");
            Require(before.Same(fixture.Snapshot()), "system owner unchanged");
        });
        Case("hardlink_refusal", () =>
        {
            using var fixture = new Fixture();
            var alias = Path.Combine(fixture.Root, "alias.json");
            if (CreateHardLink(alias, fixture.StatePath, IntPtr.Zero) == 0)
                throw new Win32Exception(Marshal.GetLastWin32Error());
            fixture.ExtraFiles.Add(alias);
            RefuseAll(fixture);
            Require(File.ReadAllBytes(alias).AsSpan().SequenceEqual(fixture.Bytes()), "alias unchanged");
        });
        Case("reparse_file_refusal", () =>
        {
            using var fixture = new Fixture();
            var retained = Path.Combine(fixture.Root, "retained.json");
            File.Move(fixture.StatePath, retained);
            fixture.ExtraFiles.Add(retained);
            File.CreateSymbolicLink(fixture.StatePath, retained);
            var before = File.ReadAllBytes(retained);
            RefuseAll(fixture);
            Require(before.AsSpan().SequenceEqual(File.ReadAllBytes(retained)), "target unchanged");
        });
        Case("reparse_directory_refusal", () =>
        {
            using var fixture = new Fixture();
            using var other = new Fixture();
            File.Delete(fixture.StatePath);
            Directory.Delete(fixture.DeviceHost);
            Directory.CreateSymbolicLink(fixture.DeviceHost, other.DeviceHost);
            var before = other.Snapshot();
            RefuseAll(fixture);
            Require(before.Same(other.Snapshot()), "other identity unchanged");
            // Unlink before the independent target fixture is disposed.
            Directory.Delete(fixture.DeviceHost);
        });
        Case("oversize_refusal", () =>
        {
            using var fixture = new Fixture();
            File.WriteAllBytes(fixture.StatePath, new byte[16385]);
            RefuseAll(fixture);
        });
        Case("untrusted_malformed_content_never_read", () =>
        {
            using var fixture = new Fixture();
            File.WriteAllBytes(fixture.StatePath, [0xff, 0]);
            WinSecurity.TestSetMetadata(fixture.StatePath,
                $"O:{user}G:BAD:P(A;;FA;;;SY)(A;;FA;;;BA)");
            RefuseAll(fixture);
        });
        Case("held_parent_namespace", ParentSharing);
        Case("held_file_read_and_delete_sharing", FileSharing);
    }

    private static void Case(string name, Action body)
    {
        currentCase = name;
        Require(clock.Elapsed < TimeSpan.FromSeconds(90), "test deadline");
        body();
        passed.Add(name);
    }

    private static void Require(bool condition, string label)
    {
        if (!condition) throw new InvalidOperationException(label);
    }

    private static void Lifecycle()
    {
        using var fixture = new Fixture(seed: false);
        fixture.Store.Prepare();
        Require(fixture.Store.Load() is null, "fresh empty");
        fixture.Store.Save(fixture.State);
        var before = fixture.Snapshot();
        Require(fixture.Store.Load() == fixture.State, "complete loaded state");
        fixture.Store.Prepare();
        Require(before.Same(fixture.Snapshot()), "read preparation noninterference");
        var legacy = fixture.State with { LocalSid = null };
        fixture.Store.Save(legacy);
        Require(fixture.Store.Load() == legacy, "schema one retained");
        var updated = fixture.State with { LastEpoch = 7 };
        fixture.Store.Save(updated);
        Require(fixture.Store.Load() == updated, "replacement full result");
        Require(Directory.GetFiles(fixture.DeviceHost).Select(Path.GetFileName)
            .SequenceEqual(new[] { "device.json" }), "no temporary residue");
        fixture.Store.Purge();
        Require(fixture.Store.Load() is null, "purged");
        fixture.Store.Purge();
        Require(fixture.Store.Load() is null, "missing idempotent");
    }

    private static void Refusal(string place, string sddl)
    {
        using var fixture = new Fixture();
        WinSecurity.TestSetMetadata(place switch
        {
            "RiAuth" => fixture.RiAuth,
            "DeviceHost" => fixture.DeviceHost,
            _ => fixture.StatePath
        }, sddl);
        RefuseAll(fixture);
    }

    private static void RefuseAll(Fixture fixture)
    {
        var before = fixture.Snapshot();
        foreach (var action in new Action[]
        {
            () => { fixture.Store.Load(); }, () => fixture.Store.Save(fixture.State),
            fixture.Store.Purge, fixture.Store.Prepare
        })
        {
            var reads = WindowsStateStore.TestContentReads;
            var decrypts = WindowsStateStore.TestDecrypts;
            var refused = false;
            try { action(); }
            catch (Exception error) when (error is InvalidDataException or Win32Exception
                or UnauthorizedAccessException or IOException) { refused = true; }
            Require(refused, "original metadata refused");
            Require(reads == WindowsStateStore.TestContentReads
                && decrypts == WindowsStateStore.TestDecrypts, "no read or decrypt");
            Require(before.Same(fixture.Snapshot()), "original owner ACL bytes namespace unchanged");
        }
    }

    private static void Joined(Action body)
    {
        Exception? failure = null;
        var worker = new Thread(() =>
        {
            try { body(); } catch (Exception error) { failure = error; }
        });
        worker.Start();
        if (!worker.Join(10000)) Environment.Exit(124);
        if (failure is not null) ExceptionDispatchInfo.Capture(failure).Throw();
    }

    private static void SharingRefusal(Action action)
    {
        var refused = false;
        try { action(); }
        catch (IOException error) when ((error.HResult & 0xffff) == 32) { refused = true; }
        catch (Win32Exception error) when (error.NativeErrorCode == 32) { refused = true; }
        Require(refused, "exact sharing violation");
    }

    // Separate ordinary-token definitions require a standard-user fixture:
    // while these handles are held, deletion/rename of ProgramData/RiAuth/DeviceHost
    // must refuse and preserve identity; foreign preplant must refuse without repair.
    // This target's worker runs as its elevated actor, not as a claimed ordinary user.
    private static void ParentSharing()
    {
        using var fixture = new Fixture();
        var before = fixture.Snapshot();
        using (var directory = WinSecurity.StateDirectory.Open(fixture.Root))
        {
            Joined(() => SharingRefusal(() => Directory.Move(fixture.RiAuth,
                Path.Combine(fixture.Root, "moved"))));
            Require(before.Same(fixture.Snapshot()), "held directory unchanged");
        }
        Require(before.Same(fixture.Snapshot()), "joined namespace unchanged");
    }

    private static void FileSharing()
    {
        using var fixture = new Fixture();
        var before = fixture.Snapshot();
        using (var directory = WinSecurity.StateDirectory.Open(fixture.Root))
        using (var file = directory.OpenState(WinSecurity.GenericRead | WinSecurity.ReadControl
            | WinSecurity.ReadAttributes, WinSecurity.ShareRead))
        {
            Require(file is not null, "held state exists");
            Joined(() => SharingRefusal(() =>
            {
                using var writer = new FileStream(fixture.StatePath, FileMode.Open,
                    FileAccess.Write, FileShare.ReadWrite | FileShare.Delete);
            }));
            Joined(() => SharingRefusal(fixture.Store.Purge));
            using var stream = new FileStream(file!, FileAccess.Read, 4096, false);
            Require(WinSecurity.ReadState(stream).AsSpan().SequenceEqual(fixture.Bytes()),
                "same held bytes");
        }
        Require(before.Same(fixture.Snapshot()), "sharing noninterference");
    }

    private sealed record Snapshot(byte[][] Metadata, byte[] State, string[] Names)
    {
        internal bool Same(Snapshot other) => Metadata.Length == other.Metadata.Length
            && Metadata.Zip(other.Metadata).All(pair => pair.First.AsSpan().SequenceEqual(pair.Second))
            && State.AsSpan().SequenceEqual(other.State) && Names.SequenceEqual(other.Names);
    }

    private sealed class Fixture : IDisposable
    {
        internal readonly string Root;
        internal string RiAuth => Path.Combine(Root, "RiAuth");
        internal string DeviceHost => Path.Combine(RiAuth, "DeviceHost");
        internal string StatePath => Path.Combine(DeviceHost, "device.json");
        internal readonly WindowsStateStore Store;
        internal readonly DeviceState State;
        internal readonly List<string> ExtraFiles = [];
        private readonly Microsoft.Win32.SafeHandles.SafeFileHandle? marker;
        private readonly Microsoft.Win32.SafeHandles.SafeFileHandle? anchor;
        private readonly byte[] rootMetadata = [];
        private bool disposed;

        internal Fixture(bool seed = true)
        {
            var drive = Path.GetPathRoot(Environment.SystemDirectory)
                ?? throw new InvalidOperationException("missing native fixture volume");
            Root = Path.Combine(drive, "riauth-state-tests-" + Guid.NewGuid().ToString("N"));
            // Exclusive native root creation; never adopt or repair a preexisting lab.
            WinSecurity.TestCreateAnchor(Root);
            try
            {
                anchor = WinSecurity.TestHoldAnchor(Root);
                rootMetadata = WinSecurity.TestSecurityBytes(Root);
                marker = WinSecurity.TestCreateMarker(Path.Combine(Root, ".fixture-owned"));
                Store = new WindowsStateStore(Root);
                State = new DeviceState(new Uri("https://identity.example.test"), "test-device",
                    "test-user", "test-user-id", "ri_windev_" + Guid.NewGuid().ToString("N"),
                    1, "S-1-5-21-1-2-3-1001");
                Store.Prepare();
                if (seed) Store.Save(State);
            }
            catch
            {
                try { Dispose(); } catch { cleanupFailed = true; }
                throw;
            }
        }

        internal byte[] Bytes() => File.Exists(StatePath) ? File.ReadAllBytes(StatePath) : [];

        internal Snapshot Snapshot()
        {
            var metadata = new[] { Root, RiAuth, DeviceHost, StatePath }
                .Where(Path.Exists).Select(WinSecurity.TestSecurityBytes).ToArray();
            var names = Directory.GetFileSystemEntries(Root)
                .Concat(Directory.GetFileSystemEntries(RiAuth))
                .Concat(Directory.GetFileSystemEntries(DeviceHost))
                .Order(StringComparer.Ordinal).ToArray();
            return new Snapshot(metadata, Bytes(), names);
        }

        public void Dispose()
        {
            if (disposed) return;
            var cleanup = Stopwatch.StartNew();
            try
            {
                Require(anchor is not null && WinSecurity.TestSameIdentity(anchor, Root), "owned root identity");
                Require(rootMetadata.AsSpan().SequenceEqual(WinSecurity.TestSecurityBytes(Root)),
                    "owned root unchanged");
                // No recursion. Unlink our synthetic reparse entry before visiting any child.
                if (Path.Exists(DeviceHost) && (File.GetAttributes(DeviceHost) & FileAttributes.ReparsePoint) != 0)
                    Directory.Delete(DeviceHost);
                else if (Directory.Exists(DeviceHost))
                {
                    WinSecurity.TestSetMetadata(DeviceHost, DirectorySddl);
                    if (Path.Exists(StatePath))
                    {
                        if ((File.GetAttributes(StatePath) & FileAttributes.ReparsePoint) == 0)
                            WinSecurity.TestSetMetadata(StatePath, FileSddl);
                        File.Delete(StatePath);
                    }
                    Require(Directory.GetFileSystemEntries(DeviceHost).Length == 0, "no unknown cleanup entries");
                    Directory.Delete(DeviceHost);
                }
                if (Directory.Exists(RiAuth))
                {
                    WinSecurity.TestSetMetadata(RiAuth, DirectorySddl);
                    Directory.Delete(RiAuth);
                }
                foreach (var path in ExtraFiles) File.Delete(path);
                if (marker is not null) WinSecurity.MarkDeleted(marker);
                marker?.Dispose();
                Require(Directory.GetFileSystemEntries(Root).Length == 0, "owned lab empty");
                anchor!.Dispose();
                Directory.Delete(Root);
                Require(!Path.Exists(Root) && cleanup.Elapsed < TimeSpan.FromSeconds(10), "cleanup bound");
                disposed = true;
            }
            catch { cleanupFailed = true; throw; }
            finally { marker?.Dispose(); anchor?.Dispose(); }
        }
    }
}
