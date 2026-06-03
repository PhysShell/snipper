using System;
using System.Threading.Tasks;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.TextManager.Interop;
using Snipper.VisualStudio;
using Xunit;

namespace Snipper.VisualStudio.IntegrationTests;

[Collection(nameof(MockedVS))]
public class TabstopInserterTests
{
    [Fact]
    public async Task InsertAsync_ReplacesTextAtCaret()
    {
        var view = new PlainTextViewStub();

        await TabstopInserter.InsertAsync(view, "public ${1:ClassName}()\n{\n    $0\n}");

        Assert.Equal(0, view.LastLine);
        Assert.Equal(0, view.LastColumn);
        Assert.Equal("public ClassName()\n{\n    \n}", view.LastText);
    }

    [Fact]
    public async Task InsertAsync_RemovesTabstopReferences()
    {
        var view = new PlainTextViewStub();

        await TabstopInserter.InsertAsync(view, "${1:Type} value = $1;$0");

        Assert.Equal("Type value = ;", view.LastText);
    }

    private sealed class PlainTextViewStub : IVsTextView
    {
        public int LastLine { get; private set; }
        public int LastColumn { get; private set; }
        public string? LastText { get; private set; }

        public int GetCaretPos(out int piLine, out int piColumn)
        {
            piLine = 0;
            piColumn = 0;
            return VSConstants.S_OK;
        }

        public int ReplaceTextOnLine(int iLine, int iStartCol, int iCharsToReplace, string pszNewText, int iNewLen)
        {
            LastLine = iLine;
            LastColumn = iStartCol;
            LastText = pszNewText;
            return VSConstants.S_OK;
        }

        public int AddCommandFilter(Microsoft.VisualStudio.OLE.Interop.IOleCommandTarget pNewCmdTarg, out Microsoft.VisualStudio.OLE.Interop.IOleCommandTarget ppNextCmdTarg) { ppNextCmdTarg = null!; return VSConstants.E_NOTIMPL; }
        public int CenterColumns(int iLine, int iLeftCol, int iColCount) => VSConstants.E_NOTIMPL;
        public int CenterLines(int iTopLine, int iCount) => VSConstants.E_NOTIMPL;
        public int ClearSelection(int fMoveToAnchor) => VSConstants.E_NOTIMPL;
        public int CloseView() => VSConstants.E_NOTIMPL;
        public int EnsureSpanVisible(TextSpan span) => VSConstants.E_NOTIMPL;
        public int GetBuffer(out IVsTextLines ppBuffer) { ppBuffer = null!; return VSConstants.E_NOTIMPL; }
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
        public IntPtr GetWindowHandle() => IntPtr.Zero;
        public int GetWordExtent(int iLine, int iCol, uint dwFlags, TextSpan[] pSpan) => VSConstants.E_NOTIMPL;
        public int HighlightMatchingBrace(uint dwFlags, uint cSpans, TextSpan[] rgBaseSpans) => VSConstants.E_NOTIMPL;
        public int Initialize(IVsTextLines pBuffer, IntPtr hwndParent, uint InitFlags, INITVIEW[] pInitView) => VSConstants.E_NOTIMPL;
        public int PositionCaretForEditing(int iLine, int cIndentLevels) => VSConstants.E_NOTIMPL;
        public int RemoveCommandFilter(Microsoft.VisualStudio.OLE.Interop.IOleCommandTarget pCmdTarg) => VSConstants.E_NOTIMPL;
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
