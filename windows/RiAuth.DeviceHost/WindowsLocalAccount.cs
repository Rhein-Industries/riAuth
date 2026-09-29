using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;

namespace RiAuth.DeviceHost;

/// <summary>Resolve only an enabled user in this machine's local SAM.</summary>
internal static class WindowsLocalAccount
{
    private const int ErrorInsufficientBuffer = 122;
    private const uint UserAccountDisabled = 0x0002;
    private const uint UserAccountLocked = 0x0010;
    private const int SidTypeUser = 1;

    [StructLayout(LayoutKind.Sequential)]
    private struct UserInfo1
    {
        internal IntPtr Name;
        internal IntPtr Password;
        internal uint PasswordAge;
        internal uint Privilege;
        internal IntPtr HomeDirectory;
        internal IntPtr Comment;
        internal uint Flags;
        internal IntPtr ScriptPath;
    }

    [DllImport("advapi32.dll", EntryPoint = "LookupAccountNameW", CharSet = CharSet.Unicode,
        SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool LookupAccountName(string? systemName, string accountName,
        byte[]? sid, ref uint sidLength, StringBuilder? domain, ref uint domainLength,
        out int use);

    [DllImport("advapi32.dll", EntryPoint = "LookupAccountSidW", CharSet = CharSet.Unicode,
        SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool LookupAccountSid(string? systemName, IntPtr sid,
        StringBuilder? name, ref uint nameLength, StringBuilder? domain,
        ref uint domainLength, out int use);

    [DllImport("advapi32.dll", EntryPoint = "ConvertSidToStringSidW", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool ConvertSidToStringSid(byte[] sid, out IntPtr text);

    [DllImport("advapi32.dll", EntryPoint = "ConvertStringSidToSidW", CharSet = CharSet.Unicode,
        SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool ConvertStringSidToSid(string text, out IntPtr sid);

    [DllImport("netapi32.dll", CharSet = CharSet.Unicode)]
    private static extern uint NetUserGetInfo(string? server, string username, uint level,
        out IntPtr buffer);

    [DllImport("netapi32.dll")]
    private static extern uint NetApiBufferFree(IntPtr buffer);

    [DllImport("kernel32.dll")]
    private static extern IntPtr LocalFree(IntPtr memory);

    internal static string ResolveEnrollment(string requested)
    {
        RequireWindows();
        var slash = requested.IndexOf('\\');
        if (slash <= 0 || slash != requested.LastIndexOf('\\') || slash == requested.Length - 1)
            throw new ArgumentException("Local account must be .\\name or MACHINE\\name");
        var domain = requested[..slash];
        var name = requested[(slash + 1)..];
        if (domain != "." && !string.Equals(domain, Environment.MachineName,
                StringComparison.OrdinalIgnoreCase))
            throw new ArgumentException("Only this machine's local account can be enrolled");
        if (name.Length is < 1 or > 64 || name.Any(c => c is '\\' or '/' or '\0' or '\r' or '\n'))
            throw new ArgumentException("Invalid local account name");
        var (sid, resolvedDomain, use) = LookupName(".\\" + name);
        if (use != SidTypeUser || !LocalDomain(resolvedDomain))
            throw new InvalidOperationException("Account is not a local Windows user");
        RequireEnabled(name);
        return sid;
    }

    internal static void RequirePinnedLocalUser(string sidText)
    {
        RequireWindows();
        if (sidText.Length is < 8 or > 256 || !sidText.StartsWith("S-", StringComparison.Ordinal))
            throw new InvalidDataException("Invalid pinned local account SID");
        if (!ConvertStringSidToSid(sidText, out var sid))
            throw new InvalidDataException("Invalid pinned local account SID");
        try
        {
            var (name, domain, use) = LookupSid(sid);
            if (use != SidTypeUser || !LocalDomain(domain))
                throw new InvalidOperationException("Pinned SID is not a local Windows user");
            RequireEnabled(name);
            var (currentSid, currentDomain, currentUse) = LookupName(".\\" + name);
            if (currentUse != SidTypeUser || !LocalDomain(currentDomain)
                || !string.Equals(currentSid, sidText, StringComparison.OrdinalIgnoreCase))
                throw new InvalidOperationException("Local account SID changed");
        }
        finally { LocalFree(sid); }
    }

    private static (string Sid, string Domain, int Use) LookupName(string account)
    {
        uint sidLength = 0, domainLength = 0;
        LookupAccountName(null, account, null, ref sidLength, null, ref domainLength, out _);
        if (Marshal.GetLastWin32Error() != ErrorInsufficientBuffer
            || sidLength is < 8 or > 1024 || domainLength > 256)
            throw new Win32Exception(Marshal.GetLastWin32Error());
        var sid = new byte[sidLength];
        var domain = new StringBuilder((int)domainLength + 1);
        if (!LookupAccountName(null, account, sid, ref sidLength, domain,
                ref domainLength, out var use))
            throw new Win32Exception(Marshal.GetLastWin32Error());
        if (!ConvertSidToStringSid(sid, out var text))
            throw new Win32Exception(Marshal.GetLastWin32Error());
        try { return (Marshal.PtrToStringUni(text) ?? throw new InvalidDataException("Empty SID"),
            domain.ToString(), use); }
        finally { LocalFree(text); }
    }

    private static (string Name, string Domain, int Use) LookupSid(IntPtr sid)
    {
        uint nameLength = 0, domainLength = 0;
        LookupAccountSid(null, sid, null, ref nameLength, null, ref domainLength, out _);
        if (Marshal.GetLastWin32Error() != ErrorInsufficientBuffer
            || nameLength is < 1 or > 256 || domainLength > 256)
            throw new Win32Exception(Marshal.GetLastWin32Error());
        var name = new StringBuilder((int)nameLength + 1);
        var domain = new StringBuilder((int)domainLength + 1);
        if (!LookupAccountSid(null, sid, name, ref nameLength, domain,
                ref domainLength, out var use))
            throw new Win32Exception(Marshal.GetLastWin32Error());
        return (name.ToString(), domain.ToString(), use);
    }

    private static void RequireEnabled(string name)
    {
        var status = NetUserGetInfo(null, name, 1, out var buffer);
        if (status != 0) throw new InvalidOperationException("Local account lookup failed");
        try
        {
            var account = Marshal.PtrToStructure<UserInfo1>(buffer);
            if ((account.Flags & (UserAccountDisabled | UserAccountLocked)) != 0)
                throw new InvalidOperationException("Local Windows account is disabled or locked");
        }
        finally { NetApiBufferFree(buffer); }
    }

    private static bool LocalDomain(string domain) => string.Equals(domain,
        Environment.MachineName, StringComparison.OrdinalIgnoreCase);

    private static void RequireWindows()
    {
        if (!OperatingSystem.IsWindows())
            throw new PlatformNotSupportedException("Local account binding requires Windows");
    }
}
