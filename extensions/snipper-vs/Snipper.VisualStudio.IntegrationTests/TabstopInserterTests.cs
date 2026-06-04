using System;
using System.IO;
using System.Threading.Tasks;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.Sdk.TestFramework;
using Microsoft.VisualStudio.Sdk.TestFramework.Xunit;
using Microsoft.VisualStudio.TextManager.Interop;
using Snipper.VisualStudio;
using Xunit;

namespace Snipper.VisualStudio.IntegrationTests;

[Collection(nameof(MockedVS))]
public class TabstopInserterTests
{
    private readonly GlobalServiceProvider serviceProvider;

    public TabstopInserterTests(GlobalServiceProvider serviceProvider)
    {
        this.serviceProvider = serviceProvider;
    }

    /// <summary>
    /// A view that does NOT implement IVsExpansionView — TabstopInserter must
    /// return cleanly without throwing.
    /// </summary>
    [VsFact]
    public async Task InsertAsync_ViewNotExpansionView_ReturnsWithoutThrowing()
    {
        await TabstopInserter.InsertAsync(new PlainTextViewStub(), "public ${1:C}() { $0 }");
    }

    /// <summary>
    /// A full expansion-capable view stub — InsertSpecificExpansion must be called
    /// exactly once and the temp snippet file must exist while the call is in flight.
    /// </summary>
    [VsFact]
    public async Task InsertAsync_ExpansionView_CallsInsertSpecificExpansion()
    {
        var mockXmlService = new CapturingXmlMemberIndexService();
        this.serviceProvider.AddService(typeof(SVsXMLMemberIndexService), mockXmlService, dispose: false);

        var view = new ExpansionViewStub();
        await TabstopInserter.InsertAsync(view, "public ${1:ClassName}()\n{\n    $0\n}");

        Assert.True(view.InsertSpecificExpansionCalled, "InsertSpecificExpansion was not called");
        Assert.NotNull(mockXmlService.LastIndexPath);
        Assert.EndsWith(".snippet", mockXmlService.LastIndexPath, StringComparison.OrdinalIgnoreCase);
    }

    [VsFact]
    public async Task InsertAsync_CreatedSnippetFile_ContainsExpectedXml()
    {
        string? capturedPath = null;
        var mockXmlService = new CapturingXmlMemberIndexService(path => capturedPath = path);
        this.serviceProvider.AddService(typeof(SVsXMLMemberIndexService), mockXmlService, dispose: false);

        await TabstopInserter.InsertAsync(
            new ExpansionViewStub(),
            "public ${1:ClassName}()\n{\n    $0\n}");

        Assert.NotNull(capturedPath);
        var xml = mockXmlService.LastIndexContent;
        Assert.NotNull(xml);
        Assert.Contains("<ID>ClassName</ID>", xml);
        Assert.Contains("$end$", xml);
    }

    // ── Stubs ────────────────────────────────────────────────────────────────

    /// <summary>IVsTextView that does NOT implement IVsExpansionView.</summary>
    private class PlainTextViewStub : IVsTextView
    {
        public int GetCaretPos(out int piLine, out int piColumn)
        { piLine = 0; piColumn = 0; return VSConstants.S_OK; }

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

    /// <summary>IVsTextView that also implements IVsExpansionView; records calls.</summary>
    private sealed class ExpansionViewStub : PlainTextViewStub, IVsExpansionView
    {
        public bool InsertSpecificExpansionCalled { get; private set; }

        public int InsertSpecificExpansion(
            IVsXMLMemberIndex pExpansion,
            TextSpan tsInsertionPoint,
            IVsExpansionClient pExpansionClient,
            Guid guidLang,
            string? pszRelativePath)
        {
            InsertSpecificExpansionCalled = true;
            return VSConstants.S_OK;
        }

        public int InsertExpansion(
            IVsExpansionClient pExpansionClient,
            string? pszExpansionTitle,
            string? pszExpansionShortcut,
            TextSpan tsInsertionPoint) => VSConstants.E_NOTIMPL;

        public int GetExpansionSpan(TextSpan[] pSpan) => VSConstants.E_NOTIMPL;
    }

    /// <summary>
    /// Minimal IVsXMLMemberIndexService that captures the snippet file path
    /// and reads its content before the temp file can be deleted.
    /// </summary>
    private sealed class CapturingXmlMemberIndexService : IVsXMLMemberIndexService
    {
        private readonly Action<string>? onIndexCreated;
        public string? LastIndexPath { get; private set; }
        public string? LastIndexContent { get; private set; }

        public CapturingXmlMemberIndexService(Action<string>? onIndexCreated = null)
        {
            this.onIndexCreated = onIndexCreated;
        }

        public int CreateXMLMemberIndex(string bstrXMLFile, out IVsXMLMemberIndex ppIndex)
        {
            LastIndexPath = bstrXMLFile;
            if (File.Exists(bstrXMLFile))
                LastIndexContent = File.ReadAllText(bstrXMLFile);
            onIndexCreated?.Invoke(bstrXMLFile);
            ppIndex = new NullMemberIndex();
            return VSConstants.S_OK;
        }

        public int GetMemberDataFromXML(string bstrXML, out IVsXMLMemberData ppObj)
        { ppObj = null!; return VSConstants.E_NOTIMPL; }

        private sealed class NullMemberIndex : IVsXMLMemberIndex
        {
            public int BuildMemberIndex() => VSConstants.S_OK;
            public int GetFilePath(out string pbstrFilePath) { pbstrFilePath = string.Empty; return VSConstants.S_OK; }
            public int GetMemberCount(out int pCount) { pCount = 0; return VSConstants.S_OK; }
            public int GetMemberIL(int iMemberIndex, out IVsXMLMemberData ppIL) { ppIL = null!; return VSConstants.E_NOTIMPL; }
            public int ParseMemberIL(string bstrMemberIL, out IVsXMLMemberData ppIL) { ppIL = null!; return VSConstants.E_NOTIMPL; }
        }
    }
}
