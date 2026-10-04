using System.ComponentModel;
using Microsoft.Win32.SafeHandles;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace RiAuth.DeviceHost;

/// <summary>Machine DPAPI plus a SYSTEM/Administrators-only ProgramData directory.</summary>
internal sealed class WindowsStateStore : IDeviceStateStore
{
    private readonly string programData;

    internal WindowsStateStore()
    {
        if (!OperatingSystem.IsWindows())
            throw new PlatformNotSupportedException("Device state requires Windows DPAPI");
        programData = Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData);
    }

#if DEVICE_STATE_TESTS
    // This constructor and the observation hooks do not exist in the host build.
    internal WindowsStateStore(string fixtureRoot) => programData = fixtureRoot;
    internal static int TestContentReads;
    internal static int TestDecrypts;
#endif

    internal void Prepare()
    {
        using var directory = WinSecurity.StateDirectory.Open(programData);
        using var state = directory.OpenState(WinSecurity.ReadControl | WinSecurity.ReadAttributes,
            WinSecurity.ShareRead);
    }

    public DeviceState? Load()
    {
        using var directory = WinSecurity.StateDirectory.Open(programData);
        using var file = directory.OpenState(WinSecurity.GenericRead | WinSecurity.ReadControl
            | WinSecurity.ReadAttributes, WinSecurity.ShareRead);
        if (file is null) return null;
#if DEVICE_STATE_TESTS
        Interlocked.Increment(ref TestContentReads);
#endif
        using var stream = new FileStream(file, FileAccess.Read, 4096, false);
        var bytes = WinSecurity.ReadState(stream);
        StoredState saved;
        try { saved = JsonSerializer.Deserialize<StoredState>(bytes)
            ?? throw new InvalidDataException("Device state is empty"); }
        catch (JsonException exception) { throw new InvalidDataException("Invalid device state", exception); }
        if (saved.Schema is not (1 or 2) || saved.Issuer is null || saved.DeviceId is null
            || saved.Username is null || saved.UserId is null || saved.ProtectedSecret is null)
            throw new InvalidDataException("Invalid device state schema");
        if (saved.Schema == 1 && saved.LocalSid is not null
            || saved.Schema == 2 && (saved.LocalSid is null
                || saved.LocalSid.Length is < 8 or > 256
                || !saved.LocalSid.StartsWith("S-", StringComparison.Ordinal)))
            throw new InvalidDataException("Invalid local account binding");
        var issuer = Protocol.Issuer(saved.Issuer);
        Protocol.Name(saved.DeviceId, "device id");
        Protocol.Name(saved.Username, "username");
        if (saved.UserId.Length is < 1 or > 128)
            throw new InvalidDataException("Invalid stored user id");
        byte[] encrypted;
        try { encrypted = Convert.FromBase64String(saved.ProtectedSecret); }
        catch (FormatException exception) { throw new InvalidDataException("Invalid protected device secret", exception); }
        if (encrypted.Length is < 1 or > 8192)
            throw new InvalidDataException("Invalid protected device secret");
#if DEVICE_STATE_TESTS
        Interlocked.Increment(ref TestDecrypts);
#endif
        var plaintext = WinSecurity.Unprotect(encrypted);
        try
        {
            var secret = Encoding.UTF8.GetString(plaintext);
            if (!secret.StartsWith("ri_windev_", StringComparison.Ordinal)
                || secret.Length is < 32 or > 1024)
                throw new InvalidDataException("Invalid stored device secret");
            return new DeviceState(issuer, saved.DeviceId, saved.Username, saved.UserId,
                secret, saved.LastEpoch, saved.LocalSid);
        }
        finally { Array.Clear(plaintext); }
    }

    public void Save(DeviceState state)
    {
        using var directory = WinSecurity.StateDirectory.Open(programData);
        using var previous = directory.OpenState(WinSecurity.ReadControl | WinSecurity.ReadAttributes,
            WinSecurity.ShareRead | WinSecurity.ShareDelete);
        var plaintext = Encoding.UTF8.GetBytes(state.DeviceSecret);
        byte[] protectedSecret;
        try { protectedSecret = WinSecurity.Protect(plaintext); }
        finally { Array.Clear(plaintext); }
        var saved = new StoredState(state.LocalSid is null ? 1 : 2, state.Issuer.AbsoluteUri,
            state.DeviceId, state.Username, state.UserId,
            Convert.ToBase64String(protectedSecret), state.LastEpoch, state.LocalSid);
        Array.Clear(protectedSecret);
        var bytes = JsonSerializer.SerializeToUtf8Bytes(saved);
        try
        {
            if (bytes.Length > 16384)
                throw new InvalidDataException("Device state is too large");
            directory.Save(bytes, previous);
        }
        finally { Array.Clear(bytes); }
    }

    public void Purge()
    {
        using var directory = WinSecurity.StateDirectory.Open(programData);
        using var file = directory.OpenState(WinSecurity.Delete | WinSecurity.ReadControl
            | WinSecurity.ReadAttributes, WinSecurity.ShareRead);
        if (file is not null) WinSecurity.MarkDeleted(file);
    }

    private sealed record StoredState(
        [property: JsonPropertyName("schema")] int Schema,
        [property: JsonPropertyName("issuer")] string? Issuer,
        [property: JsonPropertyName("device_id")] string? DeviceId,
        [property: JsonPropertyName("username")] string? Username,
        [property: JsonPropertyName("user_id")] string? UserId,
        [property: JsonPropertyName("protected_secret")] string? ProtectedSecret,
        [property: JsonPropertyName("last_epoch")] ulong? LastEpoch,
        [property: JsonPropertyName("local_sid")] string? LocalSid = null);
}

internal static class WinSecurity
{
    private const uint LocalMachine = 0x00000004;
    private const uint UiForbidden = 0x00000001;
    private const int AlreadyExists = 183;
    private const string DirectorySddl = "O:BAG:BAD:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)";
    private const string FileSddl = "O:BAG:BAD:P(A;;FA;;;SY)(A;;FA;;;BA)";

    [StructLayout(LayoutKind.Sequential)]
    private struct DataBlob { internal int Length; internal IntPtr Data; }

    [StructLayout(LayoutKind.Sequential)]
    private struct SecurityAttributes
    {
        internal int Length;
        internal IntPtr Descriptor;
        [MarshalAs(UnmanagedType.Bool)] internal bool InheritHandle;
    }

    [DllImport("crypt32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool CryptProtectData(ref DataBlob input, string? description,
        IntPtr entropy, IntPtr reserved, IntPtr prompt, uint flags, out DataBlob output);

    [DllImport("crypt32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool CryptUnprotectData(ref DataBlob input, IntPtr description,
        IntPtr entropy, IntPtr reserved, IntPtr prompt, uint flags, out DataBlob output);

    [DllImport("advapi32.dll", EntryPoint = "ConvertStringSecurityDescriptorToSecurityDescriptorW",
        SetLastError = true, CharSet = CharSet.Unicode)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool ConvertSddl(string sddl, uint revision, out IntPtr descriptor,
        out uint size);

    [DllImport("advapi32.dll", EntryPoint = "GetSecurityDescriptorOwner", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetSecurityDescriptorOwner(IntPtr descriptor, out IntPtr owner,
        [MarshalAs(UnmanagedType.Bool)] out bool defaulted);

    [DllImport("advapi32.dll", EntryPoint = "GetSecurityDescriptorDacl", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetSecurityDescriptorDacl(IntPtr descriptor,
        [MarshalAs(UnmanagedType.Bool)] out bool present, out IntPtr dacl,
        [MarshalAs(UnmanagedType.Bool)] out bool defaulted);

    [DllImport("kernel32.dll", EntryPoint = "CreateDirectoryW", SetLastError = true,
        CharSet = CharSet.Unicode)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool CreateDirectory(string path, ref SecurityAttributes attributes);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern IntPtr LocalFree(IntPtr memory);

    internal static byte[] Protect(byte[] plaintext) => Crypt(plaintext, encrypt: true);
    internal static byte[] Unprotect(byte[] ciphertext) => Crypt(ciphertext, encrypt: false);

    private static byte[] Crypt(byte[] bytes, bool encrypt)
    {
        var input = new DataBlob { Length = bytes.Length, Data = Marshal.AllocHGlobal(bytes.Length) };
        try
        {
            Marshal.Copy(bytes, 0, input.Data, bytes.Length);
            DataBlob output;
            var ok = encrypt
                ? CryptProtectData(ref input, "riAuth Windows device secret", IntPtr.Zero,
                    IntPtr.Zero, IntPtr.Zero, LocalMachine | UiForbidden, out output)
                : CryptUnprotectData(ref input, IntPtr.Zero, IntPtr.Zero,
                    IntPtr.Zero, IntPtr.Zero, UiForbidden, out output);
            if (!ok) throw new Win32Exception(Marshal.GetLastWin32Error());
            try
            {
                var result = new byte[output.Length];
                Marshal.Copy(output.Data, result, 0, result.Length);
                return result;
            }
            finally
            {
                for (var i = 0; i < output.Length; i++) Marshal.WriteByte(output.Data, i, 0);
                LocalFree(output.Data);
            }
        }
        finally
        {
            for (var i = 0; i < bytes.Length; i++) Marshal.WriteByte(input.Data, i, 0);
            Marshal.FreeHGlobal(input.Data);
        }
    }

    internal const uint GenericRead = 0x80000000;
    internal const uint GenericWrite = 0x40000000;
    internal const uint Delete = 0x00010000;
    internal const uint ReadControl = 0x00020000;
    internal const uint ReadAttributes = 0x80;
    internal const uint ShareRead = 1;
    internal const uint ShareWrite = 2;
    internal const uint ShareDelete = 4;
    private const uint OpenReparse = 0x00200000;
    private const uint BackupSemantics = 0x02000000;
    private const uint WriteThrough = 0x80000000;
    private const uint ReparseAttribute = 0x400;
    private const uint DirectoryAttribute = 0x10;
    private const int DescriptorCap = 65536;

    [StructLayout(LayoutKind.Sequential, Pack = 8)]
    private struct NativeFileTime { internal uint Low, High; }

    [StructLayout(LayoutKind.Sequential, Pack = 8)]
    private struct FileInformation
    {
        internal uint Attributes;
        internal NativeFileTime Creation, Access, Write;
        internal uint VolumeSerial, SizeHigh, SizeLow, Links, IndexHigh, IndexLow;
    }

    [StructLayout(LayoutKind.Sequential, Pack = 8)]
    private struct StandardInformation
    {
        internal long AllocationSize, EndOfFile;
        internal uint Links;
        internal byte DeletePending, Directory;
    }

    [StructLayout(LayoutKind.Sequential, Pack = 8)]
    private struct AttributeInformation { internal uint Attributes, ReparseTag; }

    [StructLayout(LayoutKind.Sequential, Pack = 8)]
    private struct AclInformation { internal uint Count, Used, Free; }

    private readonly record struct Identity(uint Volume, ulong Index);

    [DllImport("kernel32.dll", ExactSpelling = true, CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern SafeFileHandle CreateFileW(string path, uint access, uint share,
        IntPtr attributes, uint disposition, uint flags, IntPtr template);

    [DllImport("kernel32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern uint GetFileType(SafeFileHandle file);

    [DllImport("kernel32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern int GetFileInformationByHandle(SafeFileHandle file, out FileInformation info);

    [DllImport("kernel32.dll", EntryPoint = "GetFileInformationByHandleEx", ExactSpelling = true,
        SetLastError = true)]
    private static extern int GetStandardInfo(SafeFileHandle file, int infoClass,
        out StandardInformation info, uint size);

    [DllImport("kernel32.dll", EntryPoint = "GetFileInformationByHandleEx", ExactSpelling = true,
        SetLastError = true)]
    private static extern int GetAttributeInfo(SafeFileHandle file, int infoClass,
        out AttributeInformation info, uint size);

    [DllImport("kernel32.dll", ExactSpelling = true, CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern uint GetFinalPathNameByHandleW(SafeFileHandle file, StringBuilder buffer,
        uint cch, uint flags);

    [DllImport("kernel32.dll", ExactSpelling = true, CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern int GetVolumeInformationByHandleW(SafeFileHandle file, IntPtr volumeName,
        uint volumeNameCch, out uint serial, out uint maxComponentCch, out uint flags,
        StringBuilder fsName, uint fsNameCch);

    [DllImport("kernel32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern int FlushFileBuffers(SafeFileHandle file);

    [DllImport("kernel32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern int SetFileInformationByHandle(SafeFileHandle file, int infoClass,
        IntPtr buffer, uint size);

    [DllImport("advapi32.dll", ExactSpelling = true)]
    private static extern uint GetSecurityInfo(SafeFileHandle file, int objectType, uint information,
        out IntPtr owner, IntPtr group, out IntPtr dacl, IntPtr sacl, out IntPtr descriptor);

    [DllImport("advapi32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern int GetSecurityDescriptorControl(IntPtr descriptor, out ushort control,
        out uint revision);

    [DllImport("advapi32.dll", ExactSpelling = true)]
    private static extern uint GetSecurityDescriptorLength(IntPtr descriptor);

    [DllImport("advapi32.dll", ExactSpelling = true)]
    private static extern int IsValidSecurityDescriptor(IntPtr descriptor);

    [DllImport("advapi32.dll", ExactSpelling = true)]
    private static extern int IsValidAcl(IntPtr acl);

    [DllImport("advapi32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern int GetAclInformation(IntPtr acl, out AclInformation info, uint size,
        int infoClass);

    [DllImport("advapi32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern int GetAce(IntPtr acl, uint index, out IntPtr ace);

    [DllImport("advapi32.dll", ExactSpelling = true)]
    private static extern int IsValidSid(IntPtr sid);

    [DllImport("advapi32.dll", ExactSpelling = true)]
    private static extern uint GetLengthSid(IntPtr sid);

    [DllImport("advapi32.dll", ExactSpelling = true)]
    private static extern int EqualSid(IntPtr first, IntPtr second);

    [DllImport("advapi32.dll", ExactSpelling = true, CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern int ConvertStringSidToSidW(string text, out IntPtr sid);

    private static void Require(bool condition)
    {
        if (!condition) throw new InvalidDataException("Device state metadata is not admitted");
    }

    private static void Native(int result)
    {
        if (result == 0) throw new Win32Exception(Marshal.GetLastWin32Error());
    }

    private static SafeFileHandle? OpenFile(string path, uint access, uint share, bool missing)
    {
        var file = CreateFileW(path, access, share, IntPtr.Zero, 3,
            OpenReparse | BackupSemantics, IntPtr.Zero);
        if (!file.IsInvalid) return file;
        var error = Marshal.GetLastWin32Error();
        file.Dispose();
        if (missing && error == 2) return null;
        throw new Win32Exception(error);
    }

    private static string FinalPath(SafeFileHandle file)
    {
        var buffer = new StringBuilder(4096);
        var count = GetFinalPathNameByHandleW(file, buffer, 4096, 1);
        if (count == 0) throw new Win32Exception(Marshal.GetLastWin32Error());
        Require(count < 4096 && buffer.Length == count);
        return buffer.ToString();
    }

    private static Identity CheckObject(SafeFileHandle file, string expectedPath, bool directory)
    {
        Require(GetFileType(file) == 1);
        Native(GetFileInformationByHandle(file, out var info));
        Native(GetStandardInfo(file, 1, out var standard, 24));
        Native(GetAttributeInfo(file, 9, out var tag, 8));
        Require(standard.DeletePending == 0 && standard.Directory == (directory ? 1 : 0)
            && (tag.Attributes & ReparseAttribute) == 0 && tag.ReparseTag == 0
            && ((tag.Attributes & DirectoryAttribute) != 0) == directory
            && info.Attributes == tag.Attributes);
        if (!directory)
        {
            Require(standard.Links == 1 && info.Links == 1 && standard.EndOfFile >= 0
                && standard.EndOfFile <= 16384 && info.SizeHigh == 0
                && info.SizeLow == (ulong)standard.EndOfFile);
        }
        Require(string.Equals(FinalPath(file), expectedPath, StringComparison.OrdinalIgnoreCase));
        return new Identity(info.VolumeSerial, ((ulong)info.IndexHigh << 32) | info.IndexLow);
    }

    private static void CheckVolume(SafeFileHandle file)
    {
        var name = new StringBuilder(32);
        Native(GetVolumeInformationByHandleW(file, IntPtr.Zero, 0, out _, out _, out var flags,
            name, 32));
        Require(string.Equals(name.ToString(), "NTFS", StringComparison.OrdinalIgnoreCase)
            && (flags & 8) != 0 && (flags & 0x80000) == 0);
    }

    private static void Range(IntPtr root, uint length, IntPtr pointer, uint count)
    {
        var first = unchecked((ulong)root.ToInt64());
        var value = unchecked((ulong)pointer.ToInt64());
        Require(value >= first && value - first <= length && count <= length - (value - first));
    }

    private static uint SidLength(IntPtr root, uint length, IntPtr sid)
    {
        Range(root, length, sid, 8);
        var count = Marshal.ReadByte(sid, 1);
        Require(Marshal.ReadByte(sid) == 1 && count <= 15);
        var size = (uint)(8 + 4 * count);
        Range(root, length, sid, size);
        Require(IsValidSid(sid) != 0 && GetLengthSid(sid) == size);
        return size;
    }

    private static IntPtr FixedSid(string text, uint length)
    {
        Native(ConvertStringSidToSidW(text, out var sid));
        try
        {
            Require(sid != IntPtr.Zero && IsValidSid(sid) != 0 && GetLengthSid(sid) == length);
            return sid;
        }
        catch { LocalFree(sid); throw; }
    }

    private static void Admit(SafeFileHandle file, bool directory)
    {
        IntPtr descriptor = IntPtr.Zero;
        IntPtr administrators = IntPtr.Zero;
        IntPtr system = IntPtr.Zero;
        try
        {
            var result = GetSecurityInfo(file, 1, 5, out var owner, IntPtr.Zero, out var acl,
                IntPtr.Zero, out descriptor);
            if (result != 0) throw new Win32Exception((int)result);
            Require(descriptor != IntPtr.Zero && owner != IntPtr.Zero && acl != IntPtr.Zero
                && IsValidSecurityDescriptor(descriptor) != 0);
            Native(GetSecurityDescriptorControl(descriptor, out var control, out var revision));
            Require(revision == 1 && (control & 0x9004) == 0x9004);
            var length = GetSecurityDescriptorLength(descriptor);
            Require(length is >= 20 and <= DescriptorCap
                && descriptor.ToInt64() > 0 && descriptor.ToInt64() <= long.MaxValue - length);
            Require(GetSecurityDescriptorOwner(descriptor, out var checkedOwner, out _)
                && checkedOwner == owner
                && GetSecurityDescriptorDacl(descriptor, out var present, out var checkedAcl, out _)
                && present && checkedAcl == acl);
            var ownerLength = SidLength(descriptor, length, owner);
            administrators = FixedSid("S-1-5-32-544", 16);
            system = FixedSid("S-1-5-18", 12);
            Require(ownerLength == 16 && EqualSid(owner, administrators) != 0
                || ownerLength == 12 && EqualSid(owner, system) != 0);
            Range(descriptor, length, acl, 8);
            Require(Marshal.ReadByte(acl) == 2 && Marshal.ReadByte(acl, 1) == 0
                && Marshal.ReadInt16(acl, 2) == 52 && Marshal.ReadInt16(acl, 4) == 2
                && Marshal.ReadInt16(acl, 6) == 0);
            Range(descriptor, length, acl, 52);
            Require(IsValidAcl(acl) != 0);
            Native(GetAclInformation(acl, out var size, 12, 2));
            Require(size.Count == 2 && size.Used == 52 && size.Free == 0);
            var cursor = 8;
            var seenAdministrators = false;
            var seenSystem = false;
            for (uint index = 0; index < 2; index++)
            {
                Native(GetAce(acl, index, out var ace));
                Require(ace == IntPtr.Add(acl, cursor));
                Range(acl, 52, ace, 8);
                var aceSize = unchecked((ushort)Marshal.ReadInt16(ace, 2));
                Range(acl, 52, ace, aceSize);
                Require(aceSize >= 16 && (cursor & 3) == 0 && Marshal.ReadByte(ace) == 0
                    && Marshal.ReadByte(ace, 1) == (directory ? 3 : 0)
                    && Marshal.ReadInt32(ace, 4) == 0x001F01FF);
                var sid = IntPtr.Add(ace, 8);
                var sidSize = SidLength(ace, aceSize, sid);
                Require(aceSize == 8 + sidSize);
                if (sidSize == 16 && EqualSid(sid, administrators) != 0)
                {
                    Require(!seenAdministrators);
                    seenAdministrators = true;
                }
                else
                {
                    Require(sidSize == 12 && EqualSid(sid, system) != 0 && !seenSystem);
                    seenSystem = true;
                }
                cursor += aceSize;
            }
            Require(cursor == 52 && seenAdministrators && seenSystem);
        }
        finally
        {
            if (system != IntPtr.Zero) LocalFree(system);
            if (administrators != IntPtr.Zero) LocalFree(administrators);
            if (descriptor != IntPtr.Zero) LocalFree(descriptor);
        }
    }

    private static IntPtr Descriptor(string sddl)
    {
        if (!ConvertSddl(sddl, 1, out var descriptor, out _))
            throw new Win32Exception(Marshal.GetLastWin32Error());
        return descriptor;
    }

    private static void CreateDirectoryAtBirth(string path)
    {
        var descriptor = Descriptor(DirectorySddl);
        try
        {
            var attributes = new SecurityAttributes
            {
                Length = Marshal.SizeOf<SecurityAttributes>(), Descriptor = descriptor
            };
            if (!CreateDirectory(path, ref attributes))
            {
                var error = Marshal.GetLastWin32Error();
                if (error != AlreadyExists) throw new Win32Exception(error);
            }
        }
        finally { LocalFree(descriptor); }
    }

    private static SafeFileHandle CreateTemporary(string path)
    {
        var descriptor = Descriptor(FileSddl);
        var attributes = new SecurityAttributes
        {
            Length = Marshal.SizeOf<SecurityAttributes>(), Descriptor = descriptor
        };
        var memory = Marshal.AllocHGlobal(attributes.Length);
        try
        {
            Marshal.StructureToPtr(attributes, memory, false);
            var file = CreateFileW(path, GenericRead | GenericWrite | Delete | ReadControl,
                ShareRead | ShareDelete, memory, 1, OpenReparse | WriteThrough, IntPtr.Zero);
            if (!file.IsInvalid) return file;
            var error = Marshal.GetLastWin32Error();
            file.Dispose();
            throw new Win32Exception(error);
        }
        finally { Marshal.FreeHGlobal(memory); LocalFree(descriptor); }
    }

    internal static byte[] ReadState(FileStream stream)
    {
        var buffer = new byte[16385];
        try
        {
            var count = 0;
            while (count < buffer.Length)
            {
                var read = stream.Read(buffer, count, buffer.Length - count);
                if (read == 0) break;
                count += read;
            }
            Require(count <= 16384);
            return buffer.AsSpan(0, count).ToArray();
        }
        finally { Array.Clear(buffer); }
    }

    internal static void MarkDeleted(SafeFileHandle file)
    {
        var memory = Marshal.AllocHGlobal(1);
        try
        {
            Marshal.WriteByte(memory, 1);
            Native(SetFileInformationByHandle(file, 4, memory, 1));
        }
        finally { Marshal.FreeHGlobal(memory); }
    }

    private static void Rename(SafeFileHandle file, SafeFileHandle parent, bool replace)
    {
        // FILE_RENAME_INFO x64: union0, HANDLE8, DWORD16, UTF-16 name20.
        var memory = IntPtr.Zero;
        var retained = false;
        try
        {
            parent.DangerousAddRef(ref retained);
            // Use the Win32 absolute-name form; the admitted parent chain stays held.
            var name = System.IO.Path.Combine(FinalPath(parent), "device.json");
            Require(System.IO.Path.IsPathFullyQualified(name) && name.Length < 4096);
            var bytes = Encoding.Unicode.GetBytes(name + "\0");
            var size = checked(20 + bytes.Length);
            memory = Marshal.AllocHGlobal(size);
            for (var i = 0; i < size; i++) Marshal.WriteByte(memory, i, 0);
            Marshal.WriteByte(memory, replace ? (byte)1 : (byte)0);
            Marshal.WriteIntPtr(memory, 8, IntPtr.Zero);
            Marshal.WriteInt32(memory, 16, bytes.Length - 2);
            Marshal.Copy(bytes, 0, IntPtr.Add(memory, 20), bytes.Length);
            Native(SetFileInformationByHandle(file, 3, memory, (uint)size));
        }
        finally
        {
            if (retained) parent.DangerousRelease();
            if (memory != IntPtr.Zero) Marshal.FreeHGlobal(memory);
        }
    }

    internal sealed class StateDirectory : IDisposable
    {
        private sealed record Parent(SafeFileHandle File, string Path, Identity Id, bool Application);
        private readonly List<Parent> parents = [];
        private string Path => parents[^1].Path;
        private uint Volume => parents[0].Id.Volume;

        internal static StateDirectory Open(string programData)
        {
            if (!OperatingSystem.IsWindows() || IntPtr.Size != 8
                || RuntimeInformation.ProcessArchitecture != Architecture.X64)
                throw new PlatformNotSupportedException("Device state requires Windows x64 local NTFS");
            var result = new StateDirectory();
            try
            {
                Require(!string.IsNullOrWhiteSpace(programData) && System.IO.Path.IsPathFullyQualified(programData));
                using var discovery = OpenFile(programData, ReadControl | ReadAttributes,
                    ShareRead | ShareWrite, false)!;
                var final = FinalPath(discovery);
                const string prefix = @"\\?\Volume{";
                Require(final.Length > 49 && final.StartsWith(prefix, StringComparison.OrdinalIgnoreCase)
                    && final[47] == '}' && final[48] == '\\'
                    && Guid.TryParseExact(final.Substring(11, 36), "D", out _));
                var component = final[49..];
                Require(component.Length is > 0 and <= 255 && !component.Contains('\\')
                    && !component.Contains(':') && component is not ("." or ".."));
                var discoveredId = CheckObject(discovery, final, true);
                var root = final[..49];
                result.Hold(root, false);
                CheckVolume(result.parents[0].File);
                result.Hold(final, false);
                Require(result.parents[^1].Id == discoveredId);
                result.Child("RiAuth");
                result.Child("DeviceHost");
                result.ValidateParents();
                return result;
            }
            catch { result.Dispose(); throw; }
        }

        private void Hold(string path, bool application)
        {
            var file = OpenFile(path, ReadControl | ReadAttributes, ShareRead | ShareWrite, false)!;
            try
            {
                var id = CheckObject(file, path, true);
                if (parents.Count != 0) Require(id.Volume == Volume);
                if (application) Admit(file, true);
                parents.Add(new Parent(file, path, id, application));
            }
            catch { file.Dispose(); throw; }
        }

        private void Child(string component)
        {
            ValidateParents();
            var child = System.IO.Path.Combine(Path, component);
            CreateDirectoryAtBirth(child);
            Hold(child, true);
        }

        private void ValidateParents()
        {
            foreach (var parent in parents)
            {
                Require(CheckObject(parent.File, parent.Path, true) == parent.Id);
                if (parent.Application) Admit(parent.File, true);
            }
        }

        internal SafeFileHandle? OpenState(uint access, uint share)
        {
            ValidateParents();
            var path = System.IO.Path.Combine(Path, "device.json");
            var file = OpenFile(path, access, share, true);
            if (file is null) { ValidateParents(); return null; }
            try
            {
                Require(CheckObject(file, path, false).Volume == Volume);
                Admit(file, false);
                ValidateParents();
                return file;
            }
            catch { file.Dispose(); throw; }
        }

        internal void Save(byte[] bytes, SafeFileHandle? previous)
        {
            ValidateParents();
            var destination = System.IO.Path.Combine(Path, "device.json");
            var previousId = previous is null ? (Identity?)null : CheckObject(previous, destination, false);
            var temporaryPath = System.IO.Path.Combine(Path, ".device-" + Guid.NewGuid().ToString("N") + ".tmp");
            using var temporary = CreateTemporary(temporaryPath);
            // Keep the stream (and its safe handle) alive through rename and cleanup.
            FileStream? stream = null;
            var published = false;
            try
            {
                stream = new FileStream(temporary, FileAccess.ReadWrite, 4096, false);
                var temporaryId = CheckObject(temporary, temporaryPath, false);
                Require(temporaryId.Volume == Volume);
                Admit(temporary, false);
                stream.Write(bytes);
                stream.Flush();
                Native(FlushFileBuffers(temporary));
                ValidateParents();
                using (var current = OpenState(ReadControl | ReadAttributes, ShareRead | ShareDelete))
                {
                    Require(previousId is null ? current is null
                        : current is not null && CheckObject(current, destination, false) == previousId);
                    if (previous is not null) Admit(previous, false);
                    // The restricted parent protects namespace; this is not compare-and-swap.
                    Rename(temporary, parents[^1].File, previous is not null);
                    published = true;
                }
                Require(CheckObject(temporary, destination, false) == temporaryId);
                Admit(temporary, false);
                // Our held source requests WRITE|DELETE: postcheck must allow both shares.
                using var check = OpenState(ReadControl | ReadAttributes, ShareRead | ShareWrite | ShareDelete);
                Require(check is not null && CheckObject(check, destination, false) == temporaryId);
                ValidateParents();
            }
            finally
            {
                // Never remove a published destination to simulate rollback.
                try { if (!published) MarkDeleted(temporary); }
                finally { stream?.Dispose(); }
            }
        }

        public void Dispose()
        {
            for (var i = parents.Count - 1; i >= 0; i--) parents[i].File.Dispose();
            parents.Clear();
        }
    }

#if DEVICE_STATE_TESTS
    // Fixture metadata operations are excluded from every production build.
    [DllImport("advapi32.dll", EntryPoint = "SetNamedSecurityInfoW", CharSet = CharSet.Unicode)]
    private static extern uint TestSetNamedSecurityInfo(string path, uint type, uint information,
        IntPtr owner, IntPtr group, IntPtr dacl, IntPtr sacl);

    internal static void TestCreateAnchor(string path)
    {
        var descriptor = Descriptor(DirectorySddl);
        try
        {
            var attributes = new SecurityAttributes
            {
                Length = Marshal.SizeOf<SecurityAttributes>(), Descriptor = descriptor
            };
            if (!CreateDirectory(path, ref attributes))
                throw new Win32Exception(Marshal.GetLastWin32Error()); // Collision refuses.
        }
        finally { LocalFree(descriptor); }
    }

    internal static void TestSetMetadata(string path, string sddl)
    {
        var descriptor = Descriptor(sddl);
        try
        {
            var dacl = IntPtr.Zero;
            Require(GetSecurityDescriptorOwner(descriptor, out var owner, out _)
                && owner != IntPtr.Zero
                && GetSecurityDescriptorDacl(descriptor, out var present, out dacl, out _)
                && present);
            Native(GetSecurityDescriptorControl(descriptor, out var control, out _));
            var information = 5u | ((control & 0x1000) != 0 ? 0x80000000u : 0x20000000u);
            var result = TestSetNamedSecurityInfo(path, 1, information,
                owner, IntPtr.Zero, dacl, IntPtr.Zero);
            if (result != 0) throw new Win32Exception((int)result);
        }
        finally { LocalFree(descriptor); }
    }

    internal static byte[] TestSecurityBytes(string path)
    {
        using var file = OpenFile(path, ReadControl | ReadAttributes,
            ShareRead | ShareWrite | ShareDelete, false)!;
        IntPtr descriptor = IntPtr.Zero;
        try
        {
            var result = GetSecurityInfo(file, 1, 5, out var owner, IntPtr.Zero, out var acl,
                IntPtr.Zero, out descriptor);
            if (result != 0) throw new Win32Exception((int)result);
            Require(descriptor != IntPtr.Zero && IsValidSecurityDescriptor(descriptor) != 0);
            Native(GetSecurityDescriptorControl(descriptor, out var control, out var revision));
            Require(revision == 1 && (control & 0x8000) != 0);
            var length = GetSecurityDescriptorLength(descriptor);
            Require(length is >= 20 and <= DescriptorCap);
            var ownerLength = SidLength(descriptor, length, owner);
            var aclLength = 0;
            if (acl != IntPtr.Zero)
            {
                Range(descriptor, length, acl, 8);
                aclLength = unchecked((ushort)Marshal.ReadInt16(acl, 2));
                Range(descriptor, length, acl, (uint)aclLength);
            }
            var bytes = new byte[6 + ownerLength + aclLength];
            BitConverter.GetBytes(control).CopyTo(bytes, 0);
            BitConverter.GetBytes((ushort)ownerLength).CopyTo(bytes, 2);
            BitConverter.GetBytes((ushort)aclLength).CopyTo(bytes, 4);
            Marshal.Copy(owner, bytes, 6, (int)ownerLength);
            if (aclLength != 0) Marshal.Copy(acl, bytes, 6 + (int)ownerLength, aclLength);
            return bytes;
        }
        finally { if (descriptor != IntPtr.Zero) LocalFree(descriptor); }
    }

    internal static SafeFileHandle TestCreateMarker(string path) => CreateTemporary(path);

    internal static SafeFileHandle TestHoldAnchor(string path)
    {
        var file = OpenFile(path, ReadControl | ReadAttributes, ShareRead | ShareWrite, false)!;
        try
        {
            CheckObject(file, FinalPath(file), true);
            CheckVolume(file);
            Admit(file, true);
            return file;
        }
        catch { file.Dispose(); throw; }
    }

    internal static bool TestSameIdentity(SafeFileHandle held, string path)
    {
        using var current = OpenFile(path, ReadControl | ReadAttributes,
            ShareRead | ShareWrite | ShareDelete, false)!;
        Native(GetFileInformationByHandle(held, out var first));
        Native(GetFileInformationByHandle(current, out var second));
        return first.VolumeSerial == second.VolumeSerial
            && first.IndexHigh == second.IndexHigh && first.IndexLow == second.IndexLow;
    }

    internal static void TestLayouts()
    {
        Require(Marshal.SizeOf<FileInformation>() == 52
            && Marshal.OffsetOf<FileInformation>(nameof(FileInformation.VolumeSerial)).ToInt32() == 28
            && Marshal.OffsetOf<FileInformation>(nameof(FileInformation.IndexHigh)).ToInt32() == 44
            && Marshal.OffsetOf<FileInformation>(nameof(FileInformation.IndexLow)).ToInt32() == 48
            && Marshal.SizeOf<StandardInformation>() == 24
            && Marshal.OffsetOf<StandardInformation>(nameof(StandardInformation.Links)).ToInt32() == 16
            && Marshal.OffsetOf<StandardInformation>(nameof(StandardInformation.DeletePending)).ToInt32() == 20
            && Marshal.OffsetOf<StandardInformation>(nameof(StandardInformation.Directory)).ToInt32() == 21
            && Marshal.SizeOf<AttributeInformation>() == 8
            && Marshal.SizeOf<AclInformation>() == 12
            && Marshal.SizeOf<SecurityAttributes>() == 24
            && Marshal.OffsetOf<SecurityAttributes>(nameof(SecurityAttributes.Descriptor)).ToInt32() == 8
            && Marshal.OffsetOf<SecurityAttributes>(nameof(SecurityAttributes.InheritHandle)).ToInt32() == 16);
    }
#endif
}
