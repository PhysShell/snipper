using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Pipelines;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Snipper.VisualStudio;
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
        var methods = new List<string>();

        var serverTask = RunFakeServerAsync(serverRead, serverWrite, "body", methods.Add);

        await SnipperLspRpc.ExecuteCommandAsync(
            serverOutput, serverInput,
            "snipper.scaffold-constructor",
            CancellationToken.None);

        await serverTask;

        Assert.Contains("initialize", methods);
        Assert.Contains("workspace/executeCommand", methods);
        Assert.True(
            methods.IndexOf("initialize") < methods.IndexOf("workspace/executeCommand"),
            "initialize was not sent before workspace/executeCommand");
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

    private static Task RunFakeServerAsync(
        Stream readStream,
        Stream writeStream,
        string? snippetBody,
        Action<string>? onMethod = null)
    {
        return Task.Run(async () =>
        {
            while (true)
            {
                var request = await ReadMessageAsync(readStream);
                var method = request.Value<string>("method");
                Assert.False(string.IsNullOrWhiteSpace(method));
                onMethod?.Invoke(method!);

                switch (method)
                {
                    case "initialize":
                        Assert.NotNull(request["params"]);
                        await WriteResponseAsync(
                            writeStream,
                            request,
                            new JObject
                            {
                                ["capabilities"] = new JObject(),
                            });
                        break;

                    case "initialized":
                        break;

                    case "workspace/executeCommand":
                        Assert.False(string.IsNullOrWhiteSpace(request["params"]?.Value<string>("command")));
                        await WriteResponseAsync(
                            writeStream,
                            request,
                            snippetBody is null ? JValue.CreateNull() : new JValue(snippetBody));
                        break;

                    case "shutdown":
                        await WriteResponseAsync(writeStream, request, JValue.CreateNull());
                        break;

                    case "exit":
                        return;

                    default:
                        throw new InvalidOperationException($"Unexpected JSON-RPC method: {method}");
                }
            }
        });
    }

    private static async Task<JObject> ReadMessageAsync(Stream stream)
    {
        var header = await ReadHeaderAsync(stream);
        const string contentLengthPrefix = "Content-Length:";
        var contentLength = 0;

        foreach (var line in header.Split(new[] { "\r\n" }, StringSplitOptions.RemoveEmptyEntries))
        {
            if (line.StartsWith(contentLengthPrefix, StringComparison.OrdinalIgnoreCase))
                contentLength = int.Parse(line.Substring(contentLengthPrefix.Length).Trim());
        }

        Assert.True(contentLength > 0, "JSON-RPC message did not contain a Content-Length header.");

        var buffer = new byte[contentLength];
        await ReadExactlyAsync(stream, buffer);
        return JObject.Parse(Encoding.UTF8.GetString(buffer));
    }

    private static async Task<string> ReadHeaderAsync(Stream stream)
    {
        var bytes = new List<byte>();
        while (true)
        {
            var value = stream.ReadByte();
            if (value < 0)
                throw new EndOfStreamException("Stream ended while reading the JSON-RPC header.");

            bytes.Add((byte)value);
            var count = bytes.Count;
            if (count >= 4
                && bytes[count - 4] == '\r'
                && bytes[count - 3] == '\n'
                && bytes[count - 2] == '\r'
                && bytes[count - 1] == '\n')
            {
                return Encoding.ASCII.GetString(bytes.ToArray());
            }
        }
    }

    private static async Task ReadExactlyAsync(Stream stream, byte[] buffer)
    {
        var offset = 0;
        while (offset < buffer.Length)
        {
            var read = await stream.ReadAsync(buffer, offset, buffer.Length - offset);
            if (read == 0)
                throw new EndOfStreamException("Stream ended while reading the JSON-RPC body.");

            offset += read;
        }
    }

    private static Task WriteResponseAsync(Stream stream, JObject request, JToken result)
    {
        var response = new JObject
        {
            ["jsonrpc"] = "2.0",
            ["id"] = request["id"]?.DeepClone(),
            ["result"] = result,
        };

        return WriteMessageAsync(stream, response);
    }

    private static async Task WriteMessageAsync(Stream stream, JObject message)
    {
        var body = Encoding.UTF8.GetBytes(message.ToString(Formatting.None));
        var header = Encoding.ASCII.GetBytes($"Content-Length: {body.Length}\r\n\r\n");
        await stream.WriteAsync(header, 0, header.Length);
        await stream.WriteAsync(body, 0, body.Length);
        await stream.FlushAsync();
    }
}
