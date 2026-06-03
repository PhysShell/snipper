using System.Threading.Tasks;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.Shell;
using Microsoft.VisualStudio.TextManager.Interop;

namespace Snipper.VisualStudio
{
    internal static class TabstopInserter
    {
        /// <summary>
        /// Insert <paramref name="lspBody"/> as plain text at the current caret position.
        /// Must be called on the UI thread.
        /// </summary>
        public static async Task InsertAsync(IVsTextView view, string lspBody)
        {
            await ThreadHelper.JoinableTaskFactory.SwitchToMainThreadAsync();

            view.GetCaretPos(out var line, out var col);
            var text = LspSnippetConverter.ToPlainText(lspBody);
            view.ReplaceTextOnLine(line, col, 0, text, text.Length);
        }
    }
}
