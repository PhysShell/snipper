using Snipper.VisualStudio;
using Xunit;

namespace Snipper.VisualStudio.IntegrationTests;

[Collection(nameof(MockedVS))]
public class SnipperOptionsPageTests
{
    [Fact]
    public void DefaultServerPath_IsEmpty()
    {
        var page = new SnipperOptionsPage();
        Assert.Equal(string.Empty, page.ServerPath);
    }

    [Fact]
    public void DefaultRoslynPath_IsEmpty()
    {
        var page = new SnipperOptionsPage();
        Assert.Equal(string.Empty, page.RoslynPath);
    }

    [Fact]
    public void ServerPath_RoundTrips()
    {
        var page = new SnipperOptionsPage();
        page.ServerPath = @"C:\tools\snipper-lsp.exe";
        Assert.Equal(@"C:\tools\snipper-lsp.exe", page.ServerPath);
    }

    [Fact]
    public void RoslynPath_RoundTrips()
    {
        var page = new SnipperOptionsPage();
        page.RoslynPath = @"C:\tools\Snipper.Roslyn.exe";
        Assert.Equal(@"C:\tools\Snipper.Roslyn.exe", page.RoslynPath);
    }
}
