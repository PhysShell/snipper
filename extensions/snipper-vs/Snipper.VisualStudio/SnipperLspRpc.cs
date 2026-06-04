using System;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using StreamJsonRpc;

namespace Snipper.VisualStudio
{
    /// <summary>
    /// Short-lived JSON-RPC helper for per-command connections to snipper-lsp.
    /// Commands call workspace/executeCommand which is stateless — no shared document
    /// context is needed, so a fresh connection per invocation is correct and simple.
    /// </summary>
    internal static class SnipperLspRpc
    {
        /// <summary>
        /// Performs the full LSP handshake (initialize → initialized → executeCommand →
        /// shutdown → exit) on the provided streams and returns the command result.
        /// </summary>
        internal static async Task<string?> ExecuteCommandAsync(
            Stream serverOutput,
            Stream serverInput,
            string commandId,
            CancellationToken cancellationToken)
        {
            var handler = new HeaderDelimitedMessageHandler(
                serverInput, serverOutput, new JsonMessageFormatter());
            using var rpc = new JsonRpc(handler);
            rpc.StartListening();

            await rpc.InvokeWithParameterObjectAsync<object>(
                "initialize",
                new { processId = (int?)null, capabilities = new { } },
                cancellationToken);
            await rpc.NotifyWithParameterObjectAsync("initialized", new { });

            var result = await rpc.InvokeWithParameterObjectAsync<string?>(
                "workspace/executeCommand",
                new { command = commandId },
                cancellationToken);

            try
            {
                await rpc.InvokeWithParameterObjectAsync<object>(
                    "shutdown", new { }, cancellationToken);
                await rpc.NotifyWithParameterObjectAsync("exit", new { });
            }
            catch (Exception) { /* best effort on teardown */ }

            return result;
        }
    }
}
