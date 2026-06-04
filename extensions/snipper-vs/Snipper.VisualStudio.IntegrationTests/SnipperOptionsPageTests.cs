using Microsoft.VisualStudio.Sdk.TestFramework.Xunit;
using Snipper.VisualStudio;
using Xunit;

namespace Snipper.VisualStudio.IntegrationTests;

[Collection(nameof(MockedVS))]
public class SnipperOptionsPageTests
{
    [VsFact]
    public void DefaultServerPath_IsEmpty()
    {
        var page = new SnipperOptionsPage();
        Assert.Equal(string.Empty, page.ServerPath);
    }

    [VsFact]
    public void DefaultRoslynPath_IsEmpty()
    {
        var page = new SnipperOptionsPage();
        Assert.Equal(string.Empty, page.RoslynPath);
    }

    [VsFact]
    public void ServerPath_RoundTrips()
    {
        var page = new SnipperOptionsPage();
        page.ServerPath = @"C:\tools\snipper-lsp.exe";
        Assert.Equal(@"C:\tools\snipper-lsp.exe", page.ServerPath);
    }

    [VsFact]
    public void RoslynPath_RoundTrips()
    {
        var page = new SnipperOptionsPage();
        page.RoslynPath = @"C:\tools\Snipper.Roslyn.exe";
        Assert.Equal(@"C:\tools\Snipper.Roslyn.exe", page.RoslynPath);
    }
}
