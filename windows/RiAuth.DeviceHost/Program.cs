using System.Globalization;
using System.Net;
using System.Text.Json;

namespace RiAuth.DeviceHost;

internal static class Program
{
    private static async Task<int> Main(string[] args)
    {
        try
        {
            if (args.Length == 1 && args[0] == "selftest")
            {
                await SelfTest.RunAsync();
                Console.WriteLine("selftest passed");
                return 0;
            }
            if (args.Length == 0) throw new ArgumentException(Usage());
            var command = args[0];
            var options = Options.Parse(args.Skip(1).ToArray());
            using var http = new HttpClient(new SocketsHttpHandler
            {
                AllowAutoRedirect = false,
                UseCookies = false
            }) { Timeout = TimeSpan.FromSeconds(15) };
            var client = new DeviceHostClient(new DeviceApi(http), new WindowsStateStore());
            switch (command)
            {
                case "enroll":
                    options.Only("issuer", "device-id", "username", "display-name",
                        "local-account", "revision", "idempotency-key", "replace", "token-stdin");
                    options.RequireFlag("token-stdin");
                    var localSid = WindowsLocalAccount.ResolveEnrollment(
                        options.Required("local-account"));
                    await client.EnrollAsync(
                        Protocol.Issuer(options.Required("issuer")), ReadToken(),
                        options.Revision(), options.IdempotencyKey(), options.Required("device-id"),
                        options.Required("username"), options.Required("display-name"),
                        localSid, options.Flag("replace"), CancellationToken.None);
                    Console.WriteLine("device enrollment stored; disconnected access disabled");
                    break;
                case "login":
                    options.Only("issuer", "proof-stdin");
                    options.RequireFlag("proof-stdin");
                    var (password, otp) = ReadProof();
                    await client.LoginAsync(Protocol.Issuer(options.Required("issuer")),
                        password, otp, CancellationToken.None);
                    Console.WriteLine("online riAuth assertion verified; no Windows OS logon performed");
                    break;
                case "cp-account":
                    options.Only();
                    var mapped = client.Status()?.LocalSid
                        ?? throw new InvalidOperationException("Device has no local account binding; re-enroll");
                    WindowsLocalAccount.RequirePinnedLocalUser(mapped);
                    Console.Out.Write("RIAUTH-CP-ACCOUNT-V1\n" + mapped + "\n");
                    break;
                case "cp-login":
                    options.Only("proof-stdin");
                    options.RequireFlag("proof-stdin");
                    var cpState = client.Status();
                    if (cpState?.LocalSid is null)
                        throw new InvalidOperationException("Device has no local account binding; re-enroll");
                    WindowsLocalAccount.RequirePinnedLocalUser(cpState.LocalSid);
                    var (cpPassword, cpOtp) = ReadProof();
                    var approvedSid = await client.LoginAsync(cpState.Issuer, cpPassword, cpOtp,
                        CancellationToken.None);
                    if (approvedSid is null)
                        throw new InvalidOperationException("Device has no local account binding; re-enroll");
                    WindowsLocalAccount.RequirePinnedLocalUser(approvedSid);
                    Console.Out.Write("RIAUTH-CP-APPROVED-V1\n" + approvedSid + "\n");
                    break;
                case "revoke":
                    options.Only("issuer", "revision", "idempotency-key", "token-stdin");
                    options.RequireFlag("token-stdin");
                    await client.RevokeAsync(Protocol.Issuer(options.Required("issuer")),
                        ReadToken(), options.Revision(), options.IdempotencyKey(), CancellationToken.None);
                    Console.WriteLine("remote device revoked; local state purged");
                    break;
                case "purge-local":
                    options.Only("confirm-remote-revoked");
                    options.RequireFlag("confirm-remote-revoked");
                    client.PurgeAfterRemoteRevocation();
                    Console.WriteLine("local state purged after operator-confirmed remote revocation");
                    break;
                case "status":
                    options.Only();
                    var state = client.Status();
                    if (state is null) Console.WriteLine("unenrolled");
                    else Console.WriteLine(JsonSerializer.Serialize(new
                    {
                        enrolled = true,
                        issuer = state.Issuer.AbsoluteUri,
                        device_id = state.DeviceId,
                        username = state.Username,
                        user_id = state.UserId,
                        local_sid = state.LocalSid,
                        last_epoch = state.LastEpoch,
                        offline_access = false
                    }));
                    break;
                default:
                    throw new ArgumentException(Usage());
            }
            return 0;
        }
        catch (RemoteDeniedException error)
        {
            Console.Error.WriteLine(error.Message);
            return 2;
        }
        catch (HttpRequestException)
        {
            Console.Error.WriteLine("riAuth is unreachable; access denied and local state retained");
            return 3;
        }
        catch (OperationCanceledException)
        {
            Console.Error.WriteLine("riAuth timed out; access denied and local state retained");
            return 3;
        }
        catch (Exception error) when (error is ArgumentException or InvalidOperationException
            or InvalidDataException or ProtocolException or PlatformNotSupportedException
            or System.ComponentModel.Win32Exception or IOException or UnauthorizedAccessException)
        {
            Console.Error.WriteLine(error.Message);
            return 1;
        }
    }

    private static string ReadToken()
    {
        if (!Console.IsInputRedirected)
            throw new ArgumentException("Bearer token must arrive on redirected stdin");
        var token = Console.In.ReadLine();
        if (token is null || token.Length is < 1 or > 4096 || token.Any(c => c < '!' || c > '~'))
            throw new ArgumentException("Invalid bearer token on stdin");
        return token;
    }

    private static (string Password, string? Otp) ReadProof()
    {
        if (!Console.IsInputRedirected)
            throw new ArgumentException("Login proof must arrive on redirected stdin");
        var line = Console.In.ReadLine();
        if (line is null || line.Length is < 1 or > 2048)
            throw new ArgumentException("Invalid login proof on stdin");
        try
        {
            using var json = JsonDocument.Parse(line);
            var root = json.RootElement;
            if (root.ValueKind != JsonValueKind.Object || !root.TryGetProperty("password", out var password)
                || password.ValueKind != JsonValueKind.String)
                throw new ArgumentException("Login proof must be JSON with password and optional otp");
            var value = password.GetString() ?? string.Empty;
            string? otp = null;
            if (root.TryGetProperty("otp", out var code) && code.ValueKind != JsonValueKind.Null)
            {
                if (code.ValueKind != JsonValueKind.String)
                    throw new ArgumentException("Invalid login otp");
                otp = code.GetString();
            }
            foreach (var property in root.EnumerateObject())
                if (property.Name is not ("password" or "otp"))
                    throw new ArgumentException("Unexpected login proof field");
            return (value, otp);
        }
        catch (JsonException) { throw new ArgumentException("Invalid login proof JSON"); }
    }

    private static string Usage() =>
        "Usage: RiAuth.DeviceHost enroll --issuer HTTPS --device-id ID --username USER " +
        "--display-name NAME --local-account .\\USER --revision N --idempotency-key KEY " +
        "--token-stdin [--replace] | " +
        "login --issuer HTTPS --proof-stdin | revoke --issuer HTTPS --revision N " +
        "--idempotency-key KEY --token-stdin | cp-account | cp-login --proof-stdin | " +
        "purge-local --confirm-remote-revoked | status | selftest";

    private sealed class Options(Dictionary<string, string?> values)
    {
        internal static Options Parse(string[] args)
        {
            var values = new Dictionary<string, string?>(StringComparer.Ordinal);
            for (var i = 0; i < args.Length; i++)
            {
                var name = args[i];
                if (!name.StartsWith("--", StringComparison.Ordinal) || name.Length < 3)
                    throw new ArgumentException(Usage());
                name = name[2..];
                if (values.ContainsKey(name)) throw new ArgumentException("Duplicate option: " + name);
                if (name is "replace" or "token-stdin" or "proof-stdin" or "confirm-remote-revoked")
                    values.Add(name, null);
                else
                {
                    if (++i == args.Length || args[i].StartsWith("--", StringComparison.Ordinal))
                        throw new ArgumentException("Missing value for " + name);
                    values.Add(name, args[i]);
                }
            }
            return new Options(values);
        }

        internal void Only(params string[] allowed)
        {
            foreach (var key in values.Keys)
                if (!allowed.Contains(key, StringComparer.Ordinal))
                    throw new ArgumentException("Unexpected option: " + key);
        }

        internal string Required(string name) => values.TryGetValue(name, out var value)
            && !string.IsNullOrEmpty(value) ? value : throw new ArgumentException("Missing --" + name);

        internal bool Flag(string name) => values.ContainsKey(name);

        internal void RequireFlag(string name)
        {
            if (!Flag(name)) throw new ArgumentException("Missing --" + name);
        }

        internal ulong Revision() => ulong.TryParse(Required("revision"), NumberStyles.None,
            CultureInfo.InvariantCulture, out var revision) ? revision
            : throw new ArgumentException("Invalid numeric revision");

        internal string IdempotencyKey()
        {
            var key = Required("idempotency-key");
            if (key.Length > 128 || key.Any(c => c < '!' || c > '~'))
                throw new ArgumentException("Invalid idempotency key");
            return key;
        }
    }
}
