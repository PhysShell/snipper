using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.Shell;
using Microsoft.VisualStudio.TextManager.Interop;

namespace Snipper.VisualStudio
{
    internal static class TabstopInserter
    {
        private static readonly Guid CSharpLanguageGuid = new("694DD9B6-B865-4C5B-AD85-86356E9C88DC");

        /// <summary>
        /// Insert <paramref name="lspBody"/> at the current caret position with
        /// tabstop navigation active. Must be called on the UI thread.
        /// </summary>
        public static async Task InsertAsync(IVsTextView view, string lspBody)
        {
            await ThreadHelper.JoinableTaskFactory.SwitchToMainThreadAsync();

            var xml = LspSnippetConverter.ToVsSnippetXml(lspBody);
            var tempFile = Path.ChangeExtension(Path.GetTempFileName(), ".snippet");

            try
            {
                File.WriteAllText(tempFile, xml, System.Text.Encoding.UTF8);

                if (view is not IVsExpansionView expansionView)
                    return;

                // Get the IVsXMLMemberIndexService to parse the snippet file.
                if (Package.GetGlobalService(typeof(SVsXMLMemberIndexService))
                    is not IVsXMLMemberIndexService xmlService)
                    return;

                xmlService.CreateXMLMemberIndex(tempFile, out var memberIndex);
                if (memberIndex is null)
                    return;

                view.GetCaretPos(out var line, out var col);
                var insertionPoint = new TextSpan
                {
                    iStartLine = line,
                    iStartIndex = col,
                    iEndLine = line,
                    iEndIndex = col,
                };

                expansionView.InsertSpecificExpansion(
                    memberIndex,
                    insertionPoint,
                    NullExpansionClient.Instance,
                    CSharpLanguageGuid,
                    null);
            }
            finally
            {
                // Clean up asynchronously to allow the expansion session to read the file first.
                _ = Task.Delay(5000).ContinueWith(_ =>
                {
                    try { File.Delete(tempFile); } catch { /* best effort */ }
                });
            }
        }

        private sealed class NullExpansionClient : IVsExpansionClient
        {
            public static readonly NullExpansionClient Instance = new();

            public int EndExpansion() => VSConstants.S_OK;
            public int FormatSpan(IVsTextLines pBuffer, TextSpan[] ts) => VSConstants.S_OK;
            public int GetExpansionFunction(IXMLDOMNode xmlFunctionNode, string bstrFieldName, out IVsExpansionFunction pFunc)
            { pFunc = null!; return VSConstants.E_NOTIMPL; }
            public int IsValidKind(IVsTextLines pBuffer, TextSpan[] ts, string bstrKind, out int pfIsValidKind)
            { pfIsValidKind = 1; return VSConstants.S_OK; }
            public int IsValidType(IVsTextLines pBuffer, TextSpan[] ts, string[] rgTypes, int iCountTypes, out int pfIsValidType)
            { pfIsValidType = 1; return VSConstants.S_OK; }
            public int OnAfterInsertion(IVsExpansionSession pSession) => VSConstants.S_OK;
            public int OnBeforeInsertion(IVsExpansionSession pSession) => VSConstants.S_OK;
            public int OnItemChosen(string pszTitle, string pszPath) => VSConstants.S_OK;
            public int PositionCaretForEditing(IVsTextLines pBuffer, TextSpan[] ts) => VSConstants.S_OK;
        }
    }
}
