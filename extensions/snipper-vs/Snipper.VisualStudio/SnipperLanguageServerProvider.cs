using System;
using System.Collections.Generic;
using System.ComponentModel.Composition;
using System.Diagnostics;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.VisualStudio.LanguageServer.Client;
using Microsoft.VisualStudio.Threading;
using Microsoft.VisualStudio.Utilities;

namespace Snipper.VisualStudio
{
    /// <summary>
    /// Registers snipper-lsp as a language server for C# files using the stable VSSDK
    /// ILanguageClient API. This path coexists correctly with Roslyn: VS merges snippet
    /// completions with Roslyn's IntelliSense rather than routing requests exclusively
    /// to snipper-lsp (which the preview LanguageServerProvider API was doing).
    /// </summary>
    [ContentType("CSharp")]
    [Export(typeof(ILanguageClient))]
    internal sealed class SnipperLanguageClient : ILanguageClient
    {
        public string Name => "Snipper";
        public IEnumerable<string>? ConfigurationSections => null;
        public object? InitializationOptions => null;
        public IEnumerable<string>? FilesToWatch => null;
        public bool ShowNotificationOnInitializeFailed => false;

        public event AsyncEventHandler<EventArgs>? StartAsync;
        public event AsyncEventHandler<EventArgs>? StopAsync;

        public async Task<Connection?> ActivateAsync(CancellationToken token)
        {
            await Task.Yield();

            var serverPath = SnipperBinaryLocator.Resolve(
                SnipperPackage.Instance?.GetOptions()?.ServerPath);
            if (serverPath is null)
                return null;

            var psi = new ProcessStartInfo(serverPath)
            {
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                UseShellExecute = false,
                CreateNoWindow = true,
            };
            var process = Process.Start(psi)!;
            return new Connection(
                process.StandardOutput.BaseStream,
                process.StandardInput.BaseStream);
        }

        public Task OnLoadedAsync() =>
            StartAsync?.InvokeAsync(this, EventArgs.Empty) ?? Task.CompletedTask;

        public Task OnServerInitializedAsync() => Task.CompletedTask;

        public Task<InitializationFailureContext?> OnServerInitializeFailedAsync(
            ILanguageClientInitializationInfo initializationState) =>
            Task.FromResult<InitializationFailureContext?>(null);
    }
}
