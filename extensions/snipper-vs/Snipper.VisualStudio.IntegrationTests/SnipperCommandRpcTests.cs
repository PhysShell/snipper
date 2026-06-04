using System;
using System.IO;
using System.IO.Pipelines;
using System.Threading;
using System.Threading.Tasks;
using Snipper.VisualStudio;
using StreamJsonRpc;
using Xunit;

namespace Snipper.VisualStudio.IntegrationTests;

/// <summary>
/// Tests for SnipperLspRpc.ExecuteCommandAsync using an in-memory fake LSP server.
/// No real snipper-lsp binary or VS install required.
///
/// Two separate unidirectional Pipe objects (client→server and server→client) are
/// used so that swapping the stream arguments in HeaderDelimitedMessageHandler
/// immediately breaks communication — a single FullDuplexStream passed for both
/// parameters would mask the argument order entirely.
/// </summary>
public class SnipperCommandRpcTests
{
    [Fact]
    public async Task ExecuteCommandAsync_ValidCommand_ReturnsSnippetBody()
    {
        const string expectedBody = "public ${1:ClassName}()\n{\n    $0\n}";
        var (serverOutput, serverInput, serverRead, serverWrite) = CreatePipes();

        var serverTask = RunFakeServerAsync(serverRead, serverWrite, expectedBody);

        var result = await SnipperLspRpc.ExecuteCommandAsync(
            serverOutput, serverInput,
            "snipper.scaffold-constructor",
            CancellationToken.None);

        await serverTask;

        Assert.Equal(expectedBody, result);
    }

    [Fact]
    public async Task ExecuteCommandAsync_ServerReturnsNull_ReturnsNull()
    {
        var (serverOutput, serverInput, serverRead, serverWrite) = CreatePipes();

        var serverTask = RunFakeServerAsync(serverRead, serverWrite, snippetBody: null);

        var result = await SnipperLspRpc.ExecuteCommandAsync(
            serverOutput, serverInput,
            "snipper.unknown-command",
            CancellationToken.None);

        await serverTask;

        Assert.Null(result);
    }

    [Fact]
    public async Task ExecuteCommandAsync_SendsInitializeBeforeCommand()
    {
        var (serverOutput, serverInput, serverRead, serverWrite) = CreatePipes();
        bool initializeCalled = false;

        var serverTask = Task.Run(async () =>
        {
            var handler = new HeaderDelimitedMessageHandler(
                serverWrite, serverRead, new JsonMessageFormatter());
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
            serverOutput, serverInput,
            "snipper.scaffold-constructor",
            CancellationToken.None);

        await serverTask;

        Assert.True(initializeCalled, "initialize was not sent before workspace/executeCommand");
    }

    // ── Helpers ─────────────────────────────────────────────────────────────

    /// <summary>
    /// Returns four named streams from two directed pipes.
    /// serverOutput / serverInput are passed to ExecuteCommandAsync (client side).
    /// serverRead  / serverWrite are passed to the fake-server handler.
    /// </summary>
    private static (Stream serverOutput, Stream serverInput, Stream serverRead, Stream serverWrite)
        CreatePipes()
    {
        var clientToServer = new Pipe();
        var serverToClient = new Pipe();
        return (
            serverToClient.Reader.AsStream(),  // client reads responses from here
            clientToServer.Writer.AsStream(),  // client writes requests to here
            clientToServer.Reader.AsStream(),  // server reads requests from here
            serverToClient.Writer.AsStream()   // server writes responses to here
        );
    }

    private static Task RunFakeServerAsync(Stream readStream, Stream writeStream, string? snippetBody)
    {
        return Task.Run(async () =>
        {
            var handler = new HeaderDelimitedMessageHandler(
                writeStream, readStream, new JsonMessageFormatter());
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
