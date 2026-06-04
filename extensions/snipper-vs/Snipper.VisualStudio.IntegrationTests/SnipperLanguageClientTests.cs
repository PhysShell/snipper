using System;
using System.Threading;
using System.Threading.Tasks;
using Snipper.VisualStudio;
using Xunit;

namespace Snipper.VisualStudio.IntegrationTests;

/// <summary>
/// Tests for SnipperLanguageClient using plain [Fact] — no VS service mock needed
/// since ActivateAsync/OnLoadedAsync don't call into VS services during tests.
/// </summary>
public class SnipperLanguageClientTests
{
    [Fact]
    public async Task ActivateAsync_BinaryNotFound_ReturnsNull()
    {
        // SnipperPackage.Instance is null in tests → Resolve(null) tries default
        // paths but finds nothing in the test environment.
        var client = new SnipperLanguageClient();
        var conn = await client.ActivateAsync(CancellationToken.None);
        Assert.Null(conn);
    }

    [Fact]
    public async Task OnLoadedAsync_WithStartAsyncHandler_FiresEvent()
    {
        var client = new SnipperLanguageClient();
        bool fired = false;
        client.StartAsync += (_, _) => { fired = true; return Task.CompletedTask; };

        await client.OnLoadedAsync();

        Assert.True(fired);
    }

    [Fact]
    public async Task OnLoadedAsync_NoHandlers_DoesNotThrow()
    {
        var client = new SnipperLanguageClient();
        await client.OnLoadedAsync(); // must not throw even if no subscribers
    }

    [Fact]
    public async Task OnServerInitializeFailedAsync_ReturnsNull()
    {
        var client = new SnipperLanguageClient();
        var result = await client.OnServerInitializeFailedAsync(null!);
        Assert.Null(result);
    }
}
