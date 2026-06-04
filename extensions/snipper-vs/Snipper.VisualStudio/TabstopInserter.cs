using System;
using System.Threading.Tasks;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.Shell;
using Microsoft.VisualStudio.TextManager.Interop;
using MSXML;

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

            if (view is null)
                return;

            // The snippet/tabstop engine lives on the text buffer (IVsExpansion),
            // not the view. If the buffer is missing or does not support expansion
            // (e.g. a non-code view), bail out cleanly without inserting anything.
            view.GetBuffer(out var buffer);
            if (buffer is not IVsExpansion expansion)
                return;

            view.GetCaretPos(out var line, out var col);
            InsertSpecificSnippet(expansion, line, col, lspBody);
        }

        /// <summary>
        /// Core insertion logic, decoupled from <see cref="IVsTextView"/> so it can be
        /// exercised in tests with only an <see cref="IVsExpansion"/> stub. Converts the
        /// LSP snippet body to VS snippet XML, parses it into an MSXML DOM node, and asks
        /// the buffer's expansion engine to insert it with tabstop navigation.
        /// </summary>
        internal static void InsertSpecificSnippet(IVsExpansion expansion, int line, int col, string lspBody)
        {
            var snippetNode = ParseSnippetXml(LspSnippetConverter.ToVsSnippetXml(lspBody));
            if (snippetNode is null)
                return;

            var insertionPoint = new TextSpan
            {
                iStartLine = line,
                iStartIndex = col,
                iEndLine = line,
                iEndIndex = col,
            };

            expansion.InsertSpecificExpansion(
                snippetNode,
                insertionPoint,
                NullExpansionClient.Instance,
                CSharpLanguageGuid,
                null,
                out _);
        }

        /// <summary>
        /// Parses VS snippet XML into an MSXML DOM node, as required by
        /// <c>IVsExpansion.InsertSpecificExpansion</c>. The DOM object is created via its
        /// ProgID so we depend only on the <c>MSXML</c> interop interfaces shipped with the
        /// VS SDK (no MSXML primary-interop coclass reference needed). Returns null on a
        /// platform without MSXML registered (e.g. non-Windows test hosts).
        /// </summary>
        private static IXMLDOMNode? ParseSnippetXml(string xml)
        {
            var domType = Type.GetTypeFromProgID("MSXML2.DOMDocument.6.0");
            if (domType is null)
                return null;

            if (Activator.CreateInstance(domType) is not IXMLDOMDocument dom)
                return null;

            dom.loadXML(xml);

            // InsertSpecificExpansion accepts the whole snippet document; the engine
            // locates the <CodeSnippet> element within it. (IXMLDOMDocument : IXMLDOMNode.)
            return dom;
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
