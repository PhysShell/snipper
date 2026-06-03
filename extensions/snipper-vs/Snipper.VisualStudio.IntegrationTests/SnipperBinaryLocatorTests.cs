using System.IO;
using Microsoft.VisualStudio.Sdk.TestFramework.Xunit;
using Snipper.VisualStudio;
using Xunit;

namespace Snipper.VisualStudio.IntegrationTests;

[Collection(nameof(MockedVS))]
public class SnipperBinaryLocatorTests
{
    [VsFact]
    public void Resolve_ExistingConfiguredPath_ReturnsThatPath()
    {
        var tmp = Path.GetTempFileName();
        try
        {
            var result = SnipperBinaryLocator.Resolve(tmp);
            Assert.Equal(tmp, result);
        }
        finally
        {
            File.Delete(tmp);
        }
    }

    [VsFact]
    public void Resolve_NonExistentConfiguredPath_DoesNotReturn()
    {
        // Should skip the explicit path and fall through; not throw.
        var result = SnipperBinaryLocator.Resolve(@"C:\definitely\does\not\exist\snipper-lsp.exe");
        // result is null (not on PATH in test environment) or a valid PATH hit — just no throw.
    }

    [VsFact]
    public void Resolve_NullConfiguredPath_DoesNotThrow()
    {
        var _ = SnipperBinaryLocator.Resolve(null);
    }

    [VsFact]
    public void Resolve_EmptyConfiguredPath_DoesNotThrow()
    {
        var _ = SnipperBinaryLocator.Resolve(string.Empty);
    }
}
