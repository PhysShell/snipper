using System.Text.RegularExpressions;
using Snipper.VisualStudio;
using Xunit;

namespace Snipper.VisualStudio.Tests;

public class LspSnippetConverterTests
{
    [Fact]
    public void SingleTabstop_ConvertedToVsId()
    {
        var xml = LspSnippetConverter.ToVsSnippetXml("public ${1:ClassName}() { }");

        Assert.Contains("$ClassName$", xml);
        Assert.Contains("<ID>ClassName</ID>", xml);
        Assert.Contains("<Default>ClassName</Default>", xml);
    }

    [Fact]
    public void FinalCursor_ConvertedToEnd()
    {
        var xml = LspSnippetConverter.ToVsSnippetXml("foo $0 bar");

        Assert.Contains("$end$", xml);
        Assert.DoesNotContain("$0", xml);
    }

    [Fact]
    public void MultipleTabstops_AllDeclared()
    {
        var xml = LspSnippetConverter.ToVsSnippetXml("${1:Type} ${2:Name} { get; set; }");

        Assert.Contains("$Type$", xml);
        Assert.Contains("$Name$", xml);
        Assert.Contains("<ID>Type</ID>", xml);
        Assert.Contains("<ID>Name</ID>", xml);
    }

    [Fact]
    public void ScaffoldConstructorBody_FullRoundTrip()
    {
        var body = "public ${1:ClassName}()\n{\n    $0\n}";
        var xml = LspSnippetConverter.ToVsSnippetXml(body, "Scaffold constructor");

        Assert.Contains("$ClassName$", xml);
        Assert.Contains("$end$", xml);
        Assert.Contains("<ID>ClassName</ID>", xml);
        Assert.Contains("CDATA", xml);
        Assert.Contains("public $ClassName$()", xml);
    }

    [Fact]
    public void DuplicateTabstopNumber_SingleDeclaration()
    {
        // ${1:Name} appears twice — only one <Literal> should be emitted.
        var xml = LspSnippetConverter.ToVsSnippetXml("${1:Name}(${1:Name} param)");

        var count = Regex.Matches(xml, "<ID>Name</ID>").Count;
        Assert.Equal(1, count);
    }

    [Fact]
    public void NoTabstops_ValidXmlWithoutDeclarations()
    {
        var xml = LspSnippetConverter.ToVsSnippetXml("Console.WriteLine(\"hello\");");

        Assert.Contains("<CodeSnippets", xml);
        Assert.DoesNotContain("<Declarations>", xml);
    }

    [Theory]
    [InlineData("scaffoldConstructor", "scaffoldConstructor")]
    [InlineData("My Label", "MyLabel")]
    [InlineData("123abc", "123abc")]
    [InlineData("hello world!", "helloworld")]
    public void SanitizeId_StripNonAlphanumeric(string label, string expectedFragment)
    {
        // Indirectly test via placeholder with that label.
        var xml = LspSnippetConverter.ToVsSnippetXml($"${{{1}:{label}}}");

        Assert.Contains($"<ID>{expectedFragment}</ID>", xml);
    }
}
