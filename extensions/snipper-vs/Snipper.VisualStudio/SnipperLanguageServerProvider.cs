using System;
using System.Diagnostics;
using System.IO.Pipelines;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.VisualStudio.Extensibility;
using Microsoft.VisualStudio.Extensibility.LanguageServer;
using Nerdbank.Streams;

namespace Snipper.VisualStudio
{
    [VisualStudioContribution]
    internal class SnipperLanguageServerProvider : LanguageServerProvider
    {
        private readonly TraceSource logger;

        public SnipperLanguageServerProvider(
            ExtensionCore container,
            VisualStudioExtensibility extensibility,
            TraceSource traceSource)
            : base(container, traceSource)
        {
            this.logger = traceSource;
        }

        public override LanguageServerProviderConfiguration LanguageServerProviderConfiguration =>
            new("%Snipper.LanguageServer.DisplayName%",
                new[]
                {
                    new DocumentFilter { Pattern = "**/*.cs" },
                });

        public override Task<IDuplexPipe?> CreateServerConnectionAsync(CancellationToken cancellationToken)
        {
            var opts = SnipperPackage.Instance?.GetOptions();
            var serverPath = SnipperBinaryLocator.Resolve(opts?.ServerPath);

            if (serverPath is null)
            {
                this.logger.TraceEvent(TraceEventType.Error, 0,
                    "snipper-lsp binary not found. Set Snipper › Server Path in Tools › Options.");
                return Task.FromResult<IDuplexPipe?>(null);
            }

            var psi = new ProcessStartInfo(serverPath)
            {
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                UseShellExecute = false,
                CreateNoWindow = true,
            };

            var process = Process.Start(psi)!;
            var pipe = FullDuplexStream.Splice(
                process.StandardOutput.BaseStream,
                process.StandardInput.BaseStream);

            return Task.FromResult<IDuplexPipe?>(pipe);
        }

        public override Task OnServerInitializedAsync(
            LanguageServerInitializationSuccessInfo info,
            CancellationToken cancellationToken) => Task.CompletedTask;
    }
}
