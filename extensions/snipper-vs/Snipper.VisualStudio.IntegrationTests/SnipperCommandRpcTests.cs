using System;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using Nerdbank.Streams;
using Snipper.VisualStudio;
using StreamJsonRpc;
using Xunit;

namespace Snipper.VisualStudio.IntegrationTests;

/// <summary>
/// Tests for SnipperLspRpc.ExecuteCommandAsync using an in-memory fake LSP server.
/// No real snipper-lsp binary or VS install required.
/// </summary>
public class SnipperCommandRpcTests
{
    [Fact]
    public async Task ExecuteCommandAsync_ValidCommand_ReturnsSnippetBody()
    {
        const string expectedBody = "public ${1:ClassName}()\n{\n    $0\n}";
        var (clientStream, serverStream) = FullDuplexStream.CreatePair();

        var serverTask = RunFakeServerAsync(serverStream, expectedBody);

        var result = await SnipperLspRpc.ExecuteCommandAsync(
            clientStream, clientStream,
            "snipper.scaffold-constructor",
            CancellationToken.None);

        await serverTask;

        Assert.Equal(expectedBody, result);
    }

    [Fact]
    public async Task ExecuteCommandAsync_ServerReturnsNull_ReturnsNull()
    {
        var (clientStream, serverStream) = FullDuplexStream.CreatePair();

        var serverTask = RunFakeServerAsync(serverStream, snippetBody: null);

        var result = await SnipperLspRpc.ExecuteCommandAsync(
            clientStream, clientStream,
            "snipper.unknown-command",
            CancellationToken.None);

        await serverTask;

        Assert.Null(result);
    }

    [Fact]
    public async Task ExecuteCommandAsync_SendsInitializeBeforeCommand()
    {
        var (clientStream, serverStream) = FullDuplexStream.CreatePair();
        bool initializeCalled = false;

        var serverTask = Task.Run(async () =>
        {
            var handler = new HeaderDelimitedMessageHandler(
                serverStream, serverStream, new JsonMessageFormatter());
            using var rpc = new JsonRpc(handler);
            rpc.AddLocalRpcMethod("initialize",
                new Func<object?, object>(_ => { initializeCalled = true; return new { capabilities = new { } }; }));
            rpc.AddLocalRpcMethod("workspace/executeCommand",
                new Func<object?, string?>(_ => "body"));
            rpc.AddLocalRpcMethod("shutdown",
                new Func<object?, object?>(_ => null));
            rpc.AddLocalRpcMethod("initialized", new Action<object?>(_ => { }));
            rpc.AddLocalRpcMethod("exit",        new Action<object?>(_ => { }));
            rpc.StartListening();
            await rpc.Completion;
        });

        await SnipperLspRpc.ExecuteCommandAsync(
            clientStream, clientStream,
            "snipper.scaffold-constructor",
            CancellationToken.None);

        await serverTask;

        Assert.True(initializeCalled, "initialize was not sent before workspace/executeCommand");
    }

    // ── Fake server ─────────────────────────────────────────────────────────

    private static Task RunFakeServerAsync(Stream stream, string? snippetBody)
    {
        return Task.Run(async () =>
        {
            var handler = new HeaderDelimitedMessageHandler(
                stream, stream, new JsonMessageFormatter());
            using var rpc = new JsonRpc(handler);

            rpc.AddLocalRpcMethod("initialize",
                new Func<object?, object>(_ => new { capabilities = new { } }));
            rpc.AddLocalRpcMethod("workspace/executeCommand",
                new Func<object?, string?>(_ => snippetBody));
            rpc.AddLocalRpcMethod("shutdown",
                new Func<object?, object?>(_ => null));
            rpc.AddLocalRpcMethod("initialized", new Action<object?>(_ => { }));
            rpc.AddLocalRpcMethod("exit",        new Action<object?>(_ => { }));

            rpc.StartListening();
            await rpc.Completion;
        });
    }
}
