using System.Net;
using System.Text;

namespace RiAuth.DeviceHost;

internal static class SelfTest
{
    internal static async Task RunAsync()
    {
        var issuer = Protocol.Issuer("https://id.example.test/tenant");
        var state = new DeviceState(issuer, "laptop", "alice", "user-1",
            "ri_windev_" + new string('x', 40), LocalSid: "S-1-5-21-1-2-3-1001");
        var denial = new FakeHandler(request =>
            request.RequestUri!.AbsolutePath.EndsWith("/login", StringComparison.Ordinal)
                ? Reply(HttpStatusCode.Unauthorized, "{}")
                : throw new Exception("denied login must not be redeemed"));
        var deniedStore = new MemoryStore(state);
        await ExpectFailure<RemoteDeniedException>(new DeviceHostClient(
            new DeviceApi(new HttpClient(denial)), deniedStore)
            .LoginAsync(issuer, "password", null, CancellationToken.None));
        Check(denial.Redeems == 0 && denial.OfflineCalls == 0 && deniedStore.Load() == state,
            "denial must retain state and must not redeem or use offline access");

        var outage = new FakeHandler(_ => throw new HttpRequestException("offline"));
        var outageStore = new MemoryStore(state);
        await ExpectFailure<HttpRequestException>(new DeviceHostClient(
            new DeviceApi(new HttpClient(outage)), outageStore)
            .LoginAsync(issuer, "password", null, CancellationToken.None));
        Check(outage.Redeems == 0 && outage.OfflineCalls == 0 && outageStore.Load() == state,
            "network outage must retain state and must not redeem or use offline access");

        var wrongIdentity = new FakeHandler(request =>
            request.RequestUri!.AbsolutePath.EndsWith("/login", StringComparison.Ordinal)
                ? Reply(HttpStatusCode.OK,
                    "{\"signin_ticket\":\"ri_winticket_test\",\"token_type\":\"windows-signin-ticket\",\"device_id\":\"laptop\",\"username\":\"alice\",\"expires_at\":4102444800}")
                : Reply(HttpStatusCode.OK,
                    "{\"token_type\":\"windows-logon-assertion\",\"device_id\":\"laptop\",\"username\":\"alice\",\"user_id\":\"wrong-user\",\"epoch\":2,\"expires_at\":4102444800}"));
        var identityStore = new MemoryStore(state);
        await ExpectFailure<ProtocolException>(new DeviceHostClient(
            new DeviceApi(new HttpClient(wrongIdentity)), identityStore)
            .LoginAsync(issuer, "password", null, CancellationToken.None));
        Check(wrongIdentity.Redeems == 1 && identityStore.Load() == state,
            "mismatched assertion must not update accepted identity state");

        var staleEpoch = new FakeHandler(request =>
            request.RequestUri!.AbsolutePath.EndsWith("/login", StringComparison.Ordinal)
                ? Reply(HttpStatusCode.OK,
                    "{\"signin_ticket\":\"ri_winticket_test\",\"token_type\":\"windows-signin-ticket\",\"device_id\":\"laptop\",\"username\":\"alice\",\"expires_at\":4102444800}")
                : Reply(HttpStatusCode.OK,
                    "{\"token_type\":\"windows-logon-assertion\",\"device_id\":\"laptop\",\"username\":\"alice\",\"user_id\":\"user-1\",\"epoch\":2,\"expires_at\":4102444800}"));
        var epochStore = new MemoryStore(state with { LastEpoch = 3 });
        await ExpectFailure<ProtocolException>(new DeviceHostClient(
            new DeviceApi(new HttpClient(staleEpoch)), epochStore)
            .LoginAsync(issuer, "password", null, CancellationToken.None));
        Check(epochStore.Load()!.LastEpoch == 3, "epoch rollback must be denied");

        var approved = new FakeHandler(request =>
            request.RequestUri!.AbsolutePath.EndsWith("/login", StringComparison.Ordinal)
                ? Reply(HttpStatusCode.OK,
                    "{\"signin_ticket\":\"ri_winticket_test\",\"token_type\":\"windows-signin-ticket\",\"device_id\":\"laptop\",\"username\":\"alice\",\"expires_at\":4102444800}")
                : Reply(HttpStatusCode.OK,
                    "{\"token_type\":\"windows-logon-assertion\",\"device_id\":\"laptop\",\"username\":\"alice\",\"user_id\":\"user-1\",\"epoch\":2,\"expires_at\":4102444800}"));
        var approvedStore = new MemoryStore(state);
        var approvedSid = await new DeviceHostClient(new DeviceApi(new HttpClient(approved)),
            approvedStore).LoginAsync(issuer, "password", null, CancellationToken.None);
        Check(approvedSid == state.LocalSid && approved.Redeems == 1
            && approvedStore.Load()!.LastEpoch == 2,
            "only a redeemed, identity-bound assertion may release the pinned local SID");

        await StalledResponseMustTimeOutAsync(issuer, state);
    }

    private static HttpResponseMessage Reply(HttpStatusCode code, string body) =>
        new(code) { Content = new StringContent(body, Encoding.UTF8, "application/json") };

    private static async Task ExpectFailure<T>(Task action) where T : Exception
    {
        try { await action; }
        catch (T) { return; }
        throw new Exception("Expected " + typeof(T).Name);
    }

    private static void Check(bool value, string message)
    {
        if (!value) throw new Exception(message);
    }

    private sealed class MemoryStore(DeviceState state) : IDeviceStateStore
    {
        private DeviceState? saved = state;
        public DeviceState? Load() => saved;
        public void Save(DeviceState value) => saved = value;
        public void Purge() => saved = null;
    }

    private sealed class FakeHandler(Func<HttpRequestMessage, HttpResponseMessage> reply) : HttpMessageHandler
    {
        internal int Redeems { get; private set; }
        internal int OfflineCalls { get; private set; }

        protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            var path = request.RequestUri!.AbsolutePath;
            if (path.Contains("/tickets/redeem", StringComparison.Ordinal)) Redeems++;
            if (path.Contains("/offline/", StringComparison.Ordinal)) OfflineCalls++;
            return Task.FromResult(reply(request));
        }
    }

    private static async Task StalledResponseMustTimeOutAsync(Uri issuer, DeviceState state)
    {
        var stream = new StalledResponseStream();
        using var content = new ObservedStreamContent(stream);
        var requests = 0;
        var handler = new FakeHandler(request =>
        {
            requests++;
            Check(request.RequestUri!.AbsolutePath.EndsWith("/login", StringComparison.Ordinal),
                "stalled login must not dispatch another operation");
            return new HttpResponseMessage(HttpStatusCode.OK) { Content = content };
        });
        using var http = new HttpClient(handler) { Timeout = TimeSpan.FromSeconds(1) };
        var store = new ObservedStore(state);
        var login = new DeviceHostClient(new DeviceApi(http), store)
            .LoginAsync(issuer, "password", null, CancellationToken.None);
        var refusal = ExpectFailure<OperationCanceledException>(login);
        try
        {
            // This guard observes the outcome; it supplies no operation token.
            await refusal.WaitAsync(TimeSpan.FromSeconds(5));
            Check(stream.PrefixDelivered && stream.CancellationObserved,
                "partial response content must observe the request deadline");
            Check(requests == 1 && handler.Redeems == 0 && handler.OfflineCalls == 0
                && store.Saves == 0 && store.Purges == 0 && store.Load() == state
                && !login.IsCompletedSuccessfully,
                "stalled content must not approve, redeem, use offline access or change device state");
            // Check request-owned disposal before the fixture performs cleanup.
            Check(content.Disposed && stream.Disposed,
                "request failure must dispose its response content and stream");
        }
        finally
        {
            // Release an old uncancelled read only after the guard has failed.
            // Observing that task must not replace the first test failure.
            content.Dispose();
            try { await refusal.WaitAsync(TimeSpan.FromSeconds(1)); }
            catch (Exception) { }
        }
    }

    private sealed class ObservedStore(DeviceState state) : IDeviceStateStore
    {
        private DeviceState? saved = state;
        internal int Saves { get; private set; }
        internal int Purges { get; private set; }
        public DeviceState? Load() => saved;
        public void Save(DeviceState value) { Saves++; saved = value; }
        public void Purge() { Purges++; saved = null; }
    }

    private sealed class ObservedStreamContent(Stream stream) : StreamContent(stream)
    {
        internal bool Disposed { get; private set; }
        protected override void Dispose(bool disposing)
        {
            if (disposing) Disposed = true;
            base.Dispose(disposing);
        }
    }

    private sealed class StalledResponseStream : Stream
    {
        private readonly byte[] prefix = Encoding.UTF8.GetBytes("{\"signin_ticket\":");
        private readonly TaskCompletionSource<bool> disposed =
            new(TaskCreationOptions.RunContinuationsAsynchronously);
        private int offset;
        internal bool PrefixDelivered => offset == prefix.Length;
        internal bool CancellationObserved { get; private set; }
        internal bool Disposed { get; private set; }
        public override bool CanRead => !Disposed;
        public override bool CanSeek => false;
        public override bool CanWrite => false;
        public override long Length => throw new NotSupportedException();
        public override long Position
        {
            get => throw new NotSupportedException();
            set => throw new NotSupportedException();
        }
        public override void Flush() => throw new NotSupportedException();
        public override int Read(byte[] buffer, int offset, int count) => throw new NotSupportedException();
        public override long Seek(long offset, SeekOrigin origin) => throw new NotSupportedException();
        public override void SetLength(long value) => throw new NotSupportedException();
        public override void Write(byte[] buffer, int offset, int count) => throw new NotSupportedException();

        public override async ValueTask<int> ReadAsync(Memory<byte> buffer,
            CancellationToken cancellationToken = default)
        {
            if (Disposed) throw new ObjectDisposedException(nameof(StalledResponseStream));
            cancellationToken.ThrowIfCancellationRequested();
            if (buffer.IsEmpty) return 0;
            if (!PrefixDelivered)
            {
                var count = Math.Min(buffer.Length, prefix.Length - offset);
                prefix.AsMemory(offset, count).CopyTo(buffer);
                offset += count;
                return count;
            }
            try { await disposed.Task.WaitAsync(cancellationToken); }
            catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
            {
                CancellationObserved = true;
                throw;
            }
            throw new IOException("Synthetic stalled response was disposed");
        }

        protected override void Dispose(bool disposing)
        {
            if (disposing)
            {
                Disposed = true;
                disposed.TrySetResult(true);
            }
            base.Dispose(disposing);
        }
    }
}
