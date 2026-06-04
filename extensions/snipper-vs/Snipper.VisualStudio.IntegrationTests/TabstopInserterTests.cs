using System;
using System.Threading.Tasks;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.Sdk.TestFramework;
using Microsoft.VisualStudio.TextManager.Interop;
using MSXML;
using Snipper.VisualStudio;
using Xunit;

namespace Snipper.VisualStudio.IntegrationTests;

[Collection(nameof(MockedVS))]
public class TabstopInserterTests
{
    public TabstopInserterTests(GlobalServiceProvider serviceProvider)
    {
        // Initializes ThreadHelper.JoinableTaskFactory and the mocked VS main thread
        // so InsertAsync's SwitchToMainThreadAsync resolves during the test.
        serviceProvider.Reset();
    }

    /// <summary>
    /// A view whose buffer does NOT implement IVsExpansion — TabstopInserter must
    /// return cleanly without throwing.
    /// </summary>
    [Fact]
    public async Task InsertAsync_BufferNotExpansion_ReturnsWithoutThrowing()
    {
        await TabstopInserter.InsertAsync(new PlainTextViewStub(), "public ${1:C}() { $0 }");
    }

    /// <summary>
    /// The expansion engine must be asked to insert the snippet exactly once, with a
    /// non-null MSXML snippet node and the caret position as the insertion span.
    /// </summary>
    [Fact]
    public void InsertSpecificSnippet_CallsInsertSpecificExpansion_WithCaretSpan()
    {
        var expansion = new CapturingExpansion();

        TabstopInserter.InsertSpecificSnippet(expansion, line: 3, col: 7, "public ${1:ClassName}()\n{\n    $0\n}");

        Assert.True(expansion.InsertSpecificExpansionCalled, "InsertSpecificExpansion was not called");
        Assert.NotNull(expansion.LastSnippet);
        Assert.Equal(3, expansion.LastInsertionPoint.iStartLine);
        Assert.Equal(7, expansion.LastInsertionPoint.iStartIndex);
        Assert.Equal(3, expansion.LastInsertionPoint.iEndLine);
        Assert.Equal(7, expansion.LastInsertionPoint.iEndIndex);
    }

    /// <summary>
    /// The parsed snippet node round-trips the generated VS snippet XML, including the
    /// declared literal from the LSP placeholder and the $end$ final-caret marker.
    /// </summary>
    [Fact]
    public void InsertSpecificSnippet_SnippetNode_ContainsExpectedXml()
    {
        var expansion = new CapturingExpansion();

        TabstopInserter.InsertSpecificSnippet(expansion, line: 0, col: 0, "public ${1:ClassName}()\n{\n    $0\n}");

        Assert.NotNull(expansion.LastSnippet);
        var xml = expansion.LastSnippet!.xml;
        Assert.Contains("<ID>ClassName</ID>", xml);
        Assert.Contains("$end$", xml);
    }

    // ── Stubs ────────────────────────────────────────────────────────────────

    /// <summary>Records the InsertSpecificExpansion call arguments.</summary>
    private sealed class CapturingExpansion : IVsExpansion
    {
        public bool InsertSpecificExpansionCalled { get; private set; }
        public IXMLDOMNode? LastSnippet { get; private set; }
        public TextSpan LastInsertionPoint { get; private set; }

        public int InsertSpecificExpansion(
            IXMLDOMNode pSnippet,
            TextSpan tsInsertPos,
            IVsExpansionClient pExpansionClient,
            Guid guidLang,
            string pszRelativePath,
            out IVsExpansionSession pSession)
        {
            InsertSpecificExpansionCalled = true;
            LastSnippet = pSnippet;
            LastInsertionPoint = tsInsertPos;
            pSession = null!;
            return VSConstants.S_OK;
        }

        public int InsertNamedExpansion(
            string pszTitle,
            string pszPath,
            TextSpan tsInsertPos,
            IVsExpansionClient pExpansionClient,
            Guid guidLang,
            int fShowDisambiguationUI,
            out IVsExpansionSession pSession)
        { pSession = null!; return VSConstants.E_NOTIMPL; }

        public int InsertExpansion(
            TextSpan tsContext,
            TextSpan tsInsertPos,
            IVsExpansionClient pExpansionClient,
            Guid guidLang,
            out IVsExpansionSession pSession)
        { pSession = null!; return VSConstants.E_NOTIMPL; }
    }

    /// <summary>IVsTextView whose buffer is unavailable (GetBuffer returns null).</summary>
    private sealed class PlainTextViewStub : IVsTextView
    {
        public int GetCaretPos(out int piLine, out int piColumn)
        { piLine = 0; piColumn = 0; return VSConstants.S_OK; }

        public int GetBuffer(out IVsTextLines ppBuffer) { ppBuffer = null!; return VSConstants.E_NOTIMPL; }

        public int AddCommandFilter(Microsoft.VisualStudio.OLE.Interop.IOleCommandTarget pNewCmdTarg, out Microsoft.VisualStudio.OLE.Interop.IOleCommandTarget ppNextCmdTarg) { ppNextCmdTarg = null!; return VSConstants.E_NOTIMPL; }
        public int CenterColumns(int iLine, int iLeftCol, int iColCount) => VSConstants.E_NOTIMPL;
        public int CenterLines(int iTopLine, int iCount) => VSConstants.E_NOTIMPL;
        public int ClearSelection(int fMoveToAnchor) => VSConstants.E_NOTIMPL;
        public int CloseView() => VSConstants.E_NOTIMPL;
        public int EnsureSpanVisible(TextSpan span) => VSConstants.E_NOTIMPL;
        public int GetCommandFilter(Microsoft.VisualStudio.OLE.Interop.IOleCommandTarget pCurCmdTarg, out Microsoft.VisualStudio.OLE.Interop.IOleCommandTarget ppNextCmdTarg) { ppNextCmdTarg = null!; return VSConstants.E_NOTIMPL; }
        public int GetFontAndColorCategory(Guid pguidCategory, uint pdwCategoryFlags) => VSConstants.E_NOTIMPL;
        public int GetLineAndColumn(int iPos, out int piLine, out int piIndex) { piLine = 0; piIndex = 0; return VSConstants.E_NOTIMPL; }
        public int GetLineHeight(out int piLineHeight) { piLineHeight = 0; return VSConstants.E_NOTIMPL; }
        public int GetNearestPosition(int iLine, int iCol, out int piPos, out int piVirtualSpaces) { piPos = 0; piVirtualSpaces = 0; return VSConstants.E_NOTIMPL; }
        public int GetPointOfLineColumn(int iLine, int iCol, Microsoft.VisualStudio.OLE.Interop.POINT[] ppt) => VSConstants.E_NOTIMPL;
        public int GetScrollInfo(int iBar, out int piMinUnit, out int piMaxUnit, out int piVisibleUnits, out int piFirstVisibleUnit) { piMinUnit = piMaxUnit = piVisibleUnits = piFirstVisibleUnit = 0; return VSConstants.E_NOTIMPL; }
        public int GetSelectedText(out string pbstrText) { pbstrText = string.Empty; return VSConstants.E_NOTIMPL; }
        public int GetSelection(out int piAnchorLine, out int piAnchorCol, out int piEndLine, out int piEndCol) { piAnchorLine = piAnchorCol = piEndLine = piEndCol = 0; return VSConstants.E_NOTIMPL; }
        public int GetSelectionDataObject(out Microsoft.VisualStudio.OLE.Interop.IDataObject ppIDataObject) { ppIDataObject = null!; return VSConstants.E_NOTIMPL; }
        public TextSelMode GetSelectionMode() => 0;
        public int GetSelectionSpan(TextSpan[] pSpan) => VSConstants.E_NOTIMPL;
        public int GetTextStream(int iTopLine, int iTopCol, int iBottomLine, int iBottomCol, out string pbstrText) { pbstrText = string.Empty; return VSConstants.E_NOTIMPL; }
        public System.IntPtr GetWindowHandle() => System.IntPtr.Zero;
        public int GetWordExtent(int iLine, int iCol, uint dwFlags, TextSpan[] pSpan) => VSConstants.E_NOTIMPL;
        public int HighlightMatchingBrace(uint dwFlags, uint cSpans, TextSpan[] rgBaseSpans) => VSConstants.E_NOTIMPL;
        public int Initialize(IVsTextLines pBuffer, System.IntPtr hwndParent, uint InitFlags, INITVIEW[] pInitView) => VSConstants.E_NOTIMPL;
        public int PositionCaretForEditing(int iLine, int cIndentLevels) => VSConstants.E_NOTIMPL;
        public int RemoveCommandFilter(Microsoft.VisualStudio.OLE.Interop.IOleCommandTarget pCmdTarg) => VSConstants.E_NOTIMPL;
        public int ReplaceTextOnLine(int iLine, int iStartCol, int iCharsToReplace, string pszNewText, int iNewLen) => VSConstants.E_NOTIMPL;
        public int RestrictViewRange(int iMinLine, int iMaxLine, IVsViewRangeClient pClient) => VSConstants.E_NOTIMPL;
        public int SendExplicitFocus() => VSConstants.E_NOTIMPL;
        public int SetBuffer(IVsTextLines pBuffer) => VSConstants.E_NOTIMPL;
        public int SetCaretPos(int iLine, int iColumn) => VSConstants.E_NOTIMPL;
        public int SetScrollPosition(int iBar, int iFirstVisibleUnit) => VSConstants.E_NOTIMPL;
        public int SetSelection(int iAnchorLine, int iAnchorCol, int iEndLine, int iEndCol) => VSConstants.E_NOTIMPL;
        public int SetSelectionMode(TextSelMode iSelMode) => VSConstants.E_NOTIMPL;
        public int SetTopLine(int iBaseLine) => VSConstants.E_NOTIMPL;
        public int UpdateCompletionStatus(IVsCompletionSet pCompSet, uint dwFlags) => VSConstants.E_NOTIMPL;
        public int UpdateTipWindow(IVsTipWindow pTipWindow, uint dwFlags) => VSConstants.E_NOTIMPL;
        public int UpdateViewFrameCaption() => VSConstants.E_NOTIMPL;
    }
}
