using System;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.Shell;
using Microsoft.VisualStudio.Shell.Interop;
using Microsoft.VisualStudio.TextManager.Interop;

namespace Snipper.VisualStudio
{
    [PackageRegistration(UseManagedResourcesOnly = true, AllowsBackgroundLoading = true)]
    [ProvideOptionPage(typeof(SnipperOptionsPage), "Snipper", "General", 0, 0, true)]
    [Guid(PackageGuidString)]
    public sealed class SnipperPackage : AsyncPackage
    {
        public const string PackageGuidString = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";

        public static SnipperPackage? Instance { get; private set; }

        protected override async Task InitializeAsync(CancellationToken cancellationToken, IProgress<ServiceProgressData> progress)
        {
            await base.InitializeAsync(cancellationToken, progress);
            await JoinableTaskFactory.SwitchToMainThreadAsync(cancellationToken);
            Instance = this;
        }

        public SnipperOptionsPage GetOptions() =>
            (SnipperOptionsPage)GetDialogPage(typeof(SnipperOptionsPage));

        /// <summary>
        /// Insert <paramref name="lspSnippetBody"/> at the active caret position with
        /// tabstop navigation active. Must be called on the UI thread.
        /// </summary>
        public async Task InsertSnippetBodyAsync(string lspSnippetBody, CancellationToken cancellationToken)
        {
            await JoinableTaskFactory.SwitchToMainThreadAsync(cancellationToken);

            if (await GetServiceAsync(typeof(SVsTextManager)) is not IVsTextManager2 textManager)
                return;

            textManager.GetActiveView2(1, null, (uint)_VIEWFRAMETYPE.vftCodeWindow, out IVsTextView? view);
            if (view is null)
                return;

            await TabstopInserter.InsertAsync(view, lspSnippetBody);
        }
    }
}
