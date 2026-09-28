using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace RiAuth.DeviceHost;

/// <summary>Machine DPAPI plus a SYSTEM/Administrators-only ProgramData directory.</summary>
internal sealed class WindowsStateStore : IDeviceStateStore
{
    private readonly string directory;
    private readonly string path;

    internal WindowsStateStore()
    {
        if (!OperatingSystem.IsWindows())
            throw new PlatformNotSupportedException("Device state requires Windows DPAPI");
        directory = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData),
            "RiAuth", "DeviceHost");
        path = Path.Combine(directory, "device.json");
    }

    public DeviceState? Load()
    {
        SecureDirectory();
        if (!File.Exists(path)) return null;
        RejectLink(path);
        WinSecurity.SecureFile(path);
        var bytes = File.ReadAllBytes(path);
        if (bytes.Length > 16384)
            throw new InvalidDataException("Device state is too large");
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
        SecureDirectory();
        var plaintext = Encoding.UTF8.GetBytes(state.DeviceSecret);
        byte[] protectedSecret;
        try { protectedSecret = WinSecurity.Protect(plaintext); }
        finally { Array.Clear(plaintext); }
        var saved = new StoredState(state.LocalSid is null ? 1 : 2, state.Issuer.AbsoluteUri,
            state.DeviceId, state.Username, state.UserId,
            Convert.ToBase64String(protectedSecret), state.LastEpoch, state.LocalSid);
        Array.Clear(protectedSecret);
        var bytes = JsonSerializer.SerializeToUtf8Bytes(saved);
        var temporary = Path.Combine(directory, ".device-" + Guid.NewGuid().ToString("N") + ".tmp");
        try
        {
            // Close the new file before SetNamedSecurityInfo opens it for WRITE_DAC.
            using (new FileStream(temporary, FileMode.CreateNew, FileAccess.Write, FileShare.None)) { }
            WinSecurity.SecureFile(temporary);
            using (var file = new FileStream(temporary, FileMode.Open, FileAccess.Write,
                FileShare.None, 4096, FileOptions.WriteThrough))
            {
                file.Write(bytes);
                file.Flush(flushToDisk: true);
            }
            if (File.Exists(path))
            {
                RejectLink(path);
                WinSecurity.SecureFile(path);
                File.Replace(temporary, path, null);
            }
            else File.Move(temporary, path);
            WinSecurity.SecureFile(path);
        }
        finally
        {
            Array.Clear(bytes);
            if (File.Exists(temporary)) File.Delete(temporary);
        }
    }

    public void Purge()
    {
        SecureDirectory();
        if (File.Exists(path))
        {
            RejectLink(path);
            File.Delete(path);
        }
    }

    private void SecureDirectory()
    {
        var parent = Path.GetDirectoryName(directory)!;
        WinSecurity.CreateOrSecureDirectory(parent);
        WinSecurity.CreateOrSecureDirectory(directory);
        RejectLink(directory);
    }

    private static void RejectLink(string target)
    {
        if ((File.GetAttributes(target) & FileAttributes.ReparsePoint) != 0)
            throw new InvalidDataException("Device state path is a reparse point");
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
    private const uint DaclSecurityInformation = 0x00000004;
    private const uint OwnerSecurityInformation = 0x00000001;
    private const uint ProtectedDaclSecurityInformation = 0x80000000;
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

    [DllImport("advapi32.dll", EntryPoint = "SetNamedSecurityInfoW", CharSet = CharSet.Unicode)]
    private static extern uint SetNamedSecurityInfo(string path, uint objectType,
        uint information, IntPtr owner, IntPtr group, IntPtr dacl, IntPtr sacl);

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

    internal static void CreateOrSecureDirectory(string path)
    {
        var descriptor = Descriptor(DirectorySddl);
        try
        {
            var attributes = new SecurityAttributes
            {
                Length = Marshal.SizeOf<SecurityAttributes>(),
                Descriptor = descriptor
            };
            if (!CreateDirectory(path, ref attributes)
                && Marshal.GetLastWin32Error() != AlreadyExists)
                throw new Win32Exception(Marshal.GetLastWin32Error());
            if ((File.GetAttributes(path) & FileAttributes.Directory) == 0)
                throw new InvalidDataException("Device state path is not a directory");
            if ((File.GetAttributes(path) & FileAttributes.ReparsePoint) != 0)
                throw new InvalidDataException("Device state path is a reparse point");
            SetSecurity(path, descriptor);
        }
        finally { LocalFree(descriptor); }
    }

    internal static void SecureFile(string path)
    {
        var descriptor = Descriptor(FileSddl);
        try { SetSecurity(path, descriptor); }
        finally { LocalFree(descriptor); }
    }

    private static IntPtr Descriptor(string sddl)
    {
        if (!ConvertSddl(sddl, 1, out var descriptor, out _))
            throw new Win32Exception(Marshal.GetLastWin32Error());
        return descriptor;
    }

    private static void SetSecurity(string path, IntPtr descriptor)
    {
        if (!GetSecurityDescriptorOwner(descriptor, out var owner, out _) || owner == IntPtr.Zero)
            throw new Win32Exception(Marshal.GetLastWin32Error());
        if (!GetSecurityDescriptorDacl(descriptor, out var present, out var dacl, out _)
            || !present || dacl == IntPtr.Zero)
            throw new Win32Exception(Marshal.GetLastWin32Error());
        const uint fileObject = 1;
        var result = SetNamedSecurityInfo(path, fileObject,
            OwnerSecurityInformation | DaclSecurityInformation | ProtectedDaclSecurityInformation,
            owner, IntPtr.Zero, dacl, IntPtr.Zero);
        if (result != 0) throw new Win32Exception((int)result);
    }
}
