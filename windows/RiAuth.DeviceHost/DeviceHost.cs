using System.Net;
using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;

namespace RiAuth.DeviceHost;

internal sealed record DeviceState(
    Uri Issuer,
    string DeviceId,
    string Username,
    string UserId,
    string DeviceSecret,
    ulong? LastEpoch = null);

internal interface IDeviceStateStore
{
    DeviceState? Load();
    void Save(DeviceState state);
    void Purge();
}

internal sealed class RemoteDeniedException(HttpStatusCode status)
    : Exception($"riAuth rejected the request ({(int)status})");

internal sealed class ProtocolException(string message) : Exception(message);

internal static class Protocol
{
    internal static string RequiredString(JsonElement value, string name)
    {
        if (!value.TryGetProperty(name, out var child) || child.ValueKind != JsonValueKind.String)
            throw new ProtocolException($"Invalid riAuth {name} response");
        var result = child.GetString();
        if (string.IsNullOrEmpty(result))
            throw new ProtocolException($"Invalid riAuth {name} response");
        return result;
    }

    internal static JsonElement RequiredObject(JsonElement value, string name)
    {
        if (!value.TryGetProperty(name, out var child) || child.ValueKind != JsonValueKind.Object)
            throw new ProtocolException($"Invalid riAuth {name} response");
        return child;
    }

    internal static void Equal(string actual, string expected, string name)
    {
        if (!string.Equals(actual, expected, StringComparison.Ordinal))
            throw new ProtocolException($"riAuth {name} does not match the enrolled device");
    }

    internal static Uri Issuer(string raw)
    {
        if (!Uri.TryCreate(raw, UriKind.Absolute, out var uri)
            || uri.Scheme != Uri.UriSchemeHttps
            || string.IsNullOrEmpty(uri.Host)
            || !string.IsNullOrEmpty(uri.UserInfo)
            || !string.IsNullOrEmpty(uri.Query)
            || !string.IsNullOrEmpty(uri.Fragment))
            throw new ArgumentException("Issuer must be an HTTPS URL without credentials, query, or fragment");
        return new Uri(uri.AbsoluteUri.TrimEnd('/'));
    }

    internal static void Name(string value, string label)
    {
        if (value.Length is < 1 or > 64 || !value.All(c =>
                c is >= 'a' and <= 'z' or >= 'A' and <= 'Z' or >= '0' and <= '9'
                    or '.' or '-' or '_' or '@'))
            throw new ArgumentException($"Invalid {label}");
    }
}

internal sealed class DeviceApi(HttpClient http)
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower
    };

    private async Task<JsonDocument> SendAsync(
        Uri issuer,
        HttpMethod method,
        string path,
        object? body,
        string? bearer,
        ulong? revision,
        string? idempotencyKey,
        CancellationToken cancellation)
    {
        using var request = new HttpRequestMessage(method, issuer.AbsoluteUri.TrimEnd('/') + path);
        if (bearer is not null)
            request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", bearer);
        if (revision is not null)
            request.Headers.TryAddWithoutValidation("If-Match", $"\"{revision}\"");
        if (idempotencyKey is not null)
            request.Headers.TryAddWithoutValidation("Idempotency-Key", idempotencyKey);
        if (body is not null)
            request.Content = new StringContent(JsonSerializer.Serialize(body, JsonOptions), Encoding.UTF8, "application/json");
        using var response = await http.SendAsync(request, HttpCompletionOption.ResponseHeadersRead, cancellation);
        if (!response.IsSuccessStatusCode)
            throw new RemoteDeniedException(response.StatusCode);
        await using var stream = await response.Content.ReadAsStreamAsync(cancellation);
        using var limited = new MemoryStream();
        var buffer = new byte[8192];
        while (true)
        {
            var count = await stream.ReadAsync(buffer, cancellation);
            if (count == 0) break;
            if (limited.Length + count > 65536)
                throw new ProtocolException("riAuth response exceeds 64 KiB");
            limited.Write(buffer, 0, count);
        }
        limited.Position = 0;
        try { return await JsonDocument.ParseAsync(limited, cancellationToken: cancellation); }
        catch (JsonException) { throw new ProtocolException("Invalid riAuth JSON response"); }
    }

    internal Task<JsonDocument> EnrollAsync(Uri issuer, string token, ulong revision, string key,
        string id, string username, string displayName, CancellationToken cancellation) =>
        SendAsync(issuer, HttpMethod.Post, "/api/windows-devices",
            new { id, display_name = displayName, username }, token, revision, key, cancellation);

    internal Task<JsonDocument> RevokeAsync(Uri issuer, string token, ulong revision, string key,
        string id, CancellationToken cancellation) =>
        SendAsync(issuer, HttpMethod.Delete,
            "/api/windows-devices/" + Uri.EscapeDataString(id), null, token, revision, key, cancellation);

    internal Task<JsonDocument> LoginAsync(DeviceState state, string password, string? otp,
        CancellationToken cancellation) =>
        SendAsync(state.Issuer, HttpMethod.Post, "/api/windows-devices/login",
            new { device_id = state.DeviceId, device_secret = state.DeviceSecret,
                username = state.Username, password, otp }, null, null, null, cancellation);

    internal Task<JsonDocument> RedeemAsync(DeviceState state, string ticket,
        CancellationToken cancellation) =>
        SendAsync(state.Issuer, HttpMethod.Post, "/api/windows-devices/tickets/redeem",
            new { ticket }, null, null, null, cancellation);
}

internal sealed class DeviceHostClient(DeviceApi api, IDeviceStateStore store)
{
    internal DeviceState? Status() => store.Load();

    internal async Task EnrollAsync(Uri issuer, string token, ulong revision, string key,
        string id, string username, string displayName, bool replace, CancellationToken cancellation)
    {
        Protocol.Name(id, "device id");
        Protocol.Name(username, "username");
        if (displayName.Length is < 1 or > 200 || displayName.Any(char.IsControl))
            throw new ArgumentException("Invalid display name");
        var previous = store.Load();
        if (previous is not null && !replace)
            throw new InvalidOperationException("Device state exists; use --replace for explicit rotation");
        if (previous is not null && (previous.Issuer != issuer || previous.DeviceId != id))
            throw new InvalidOperationException("Existing device belongs to another issuer or id; revoke first");
        using var result = await api.EnrollAsync(issuer, token, revision, key, id, username, displayName, cancellation);
        var root = result.RootElement;
        var device = Protocol.RequiredObject(root, "device");
        Protocol.Equal(Protocol.RequiredString(device, "id"), id, "device id");
        Protocol.Equal(Protocol.RequiredString(device, "username"), username, "username");
        if (!device.TryGetProperty("revoked", out var revoked) || revoked.ValueKind != JsonValueKind.False)
            throw new ProtocolException("Enrolled device is revoked");
        if (root.TryGetProperty("offline_ticket", out var offline) && offline.ValueKind != JsonValueKind.Null)
            throw new ProtocolException("Unexpected offline ticket: disconnected access is disabled");
        var userId = Protocol.RequiredString(device, "user_id");
        var secret = Protocol.RequiredString(root, "device_secret");
        if (!secret.StartsWith("ri_windev_", StringComparison.Ordinal) || secret.Length is < 32 or > 1024)
            throw new ProtocolException("Invalid device secret from riAuth");
        store.Save(new DeviceState(issuer, id, username, userId, secret));
    }

    internal async Task LoginAsync(Uri issuer, string password, string? otp, CancellationToken cancellation)
    {
        if (password.Length is < 1 or > 1024 || otp?.Length > 128)
            throw new ArgumentException("Invalid login proof");
        var state = store.Load() ?? throw new InvalidOperationException("Device is not enrolled");
        if (state.Issuer != issuer)
            throw new InvalidOperationException("Issuer differs from enrolled device");
        // Every decision requires the server. There is deliberately no offline path.
        using var login = await api.LoginAsync(state, password, otp, cancellation);
        var loginRoot = login.RootElement;
        Protocol.Equal(Protocol.RequiredString(loginRoot, "token_type"), "windows-signin-ticket", "ticket type");
        Protocol.Equal(Protocol.RequiredString(loginRoot, "device_id"), state.DeviceId, "device id");
        Protocol.Equal(Protocol.RequiredString(loginRoot, "username"), state.Username, "username");
        if (!loginRoot.TryGetProperty("expires_at", out var ticketExpiry)
            || !ticketExpiry.TryGetInt64(out var ticketExpiresAt)
            || ticketExpiresAt <= DateTimeOffset.UtcNow.ToUnixTimeSeconds())
            throw new ProtocolException("Expired riAuth sign-in ticket");
        var ticket = Protocol.RequiredString(loginRoot, "signin_ticket");
        if (!ticket.StartsWith("ri_winticket_", StringComparison.Ordinal) || ticket.Length > 128)
            throw new ProtocolException("Invalid sign-in ticket from riAuth");
        using var redeemed = await api.RedeemAsync(state, ticket, cancellation);
        var assertion = redeemed.RootElement;
        Protocol.Equal(Protocol.RequiredString(assertion, "token_type"), "windows-logon-assertion", "assertion type");
        Protocol.Equal(Protocol.RequiredString(assertion, "device_id"), state.DeviceId, "device id");
        Protocol.Equal(Protocol.RequiredString(assertion, "username"), state.Username, "username");
        Protocol.Equal(Protocol.RequiredString(assertion, "user_id"), state.UserId, "user id");
        if (!assertion.TryGetProperty("epoch", out var epochValue)
            || !epochValue.TryGetUInt64(out var epoch)
            || state.LastEpoch is not null && epoch < state.LastEpoch)
            throw new ProtocolException("Invalid or stale riAuth user epoch");
        if (!assertion.TryGetProperty("expires_at", out var expiry)
            || !expiry.TryGetInt64(out var expiresAt)
            || expiresAt <= DateTimeOffset.UtcNow.ToUnixTimeSeconds()
            || expiresAt > ticketExpiresAt)
            throw new ProtocolException("Expired riAuth assertion");
        store.Save(state with { LastEpoch = epoch });
    }

    internal async Task RevokeAsync(Uri issuer, string token, ulong revision, string key, CancellationToken cancellation)
    {
        var state = store.Load() ?? throw new InvalidOperationException("Device is not enrolled");
        if (state.Issuer != issuer)
            throw new InvalidOperationException("Issuer differs from enrolled device");
        using var result = await api.RevokeAsync(state.Issuer, token, revision, key, state.DeviceId, cancellation);
        var root = result.RootElement;
        Protocol.Equal(Protocol.RequiredString(root, "id"), state.DeviceId, "device id");
        if (!root.TryGetProperty("revoked", out var revoked) || revoked.ValueKind != JsonValueKind.True)
            throw new ProtocolException("riAuth did not confirm revocation");
        store.Purge();
    }

    internal void PurgeAfterRemoteRevocation() => store.Purge();
}
