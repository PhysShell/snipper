using System.Xml;
using Snipper.VisualStudio;
using Xunit;

namespace Snipper.VisualStudio.IntegrationTests;

/// <summary>
/// Run LspSnippetConverter on the mock VS UI thread to catch any threading
/// or COM-affinity assumptions we might accidentally introduce in future.
/// </summary>
[Collection(nameof(MockedVS))]
public class LspSnippetConverterVsTests
{
    [Fact]
    public void Output_IsWellFormedXml()
    {
        var xml = LspSnippetConverter.ToVsSnippetXml("public ${1:ClassName}()\n{\n    $0\n}");

        var doc = new XmlDocument();
        doc.LoadXml(xml); // throws XmlException if malformed
        Assert.Equal("CodeSnippets", doc.DocumentElement!.LocalName);
    }

    [Fact]
    public void Output_ContainsCorrectSchemaNamespace()
    {
        var xml = LspSnippetConverter.ToVsSnippetXml("foo $0");

        Assert.Contains("http://schemas.microsoft.com/VisualStudio/2005/CodeSnippet", xml);
    }

    [Fact]
    public void ScaffoldConstructor_DeclaresClassNameLiteral()
    {
        var body = "public ${1:ClassName}()\n{\n    $0\n}";
        var xml = LspSnippetConverter.ToVsSnippetXml(body);

        var doc = new XmlDocument();
        var ns = new XmlNamespaceManager(doc.NameTable);
        ns.AddNamespace("cs", "http://schemas.microsoft.com/VisualStudio/2005/CodeSnippet");
        doc.LoadXml(xml);

        var literal = doc.SelectSingleNode("//cs:Literal/cs:ID[text()='ClassName']", ns);
        Assert.NotNull(literal);
    }

    [Fact]
    public void ImplementInterface_AllThreeLiterals()
    {
        var body = "public ${1:ReturnType} ${2:MethodName}(${3:params})\n{\n    throw new System.NotImplementedException();\n}";
        var xml = LspSnippetConverter.ToVsSnippetXml(body);

        var doc = new XmlDocument();
        var ns = new XmlNamespaceManager(doc.NameTable);
        ns.AddNamespace("cs", "http://schemas.microsoft.com/VisualStudio/2005/CodeSnippet");
        doc.LoadXml(xml);

        var literals = doc.SelectNodes("//cs:Literal/cs:ID", ns)!;
        Assert.Equal(3, literals.Count);
        Assert.Equal("ReturnType", literals[0]!.InnerText);
        Assert.Equal("MethodName", literals[1]!.InnerText);
        Assert.Equal("params",     literals[2]!.InnerText);
    }

    [Fact]
    public void CodeBody_ContainsCdataSection()
    {
        var xml = LspSnippetConverter.ToVsSnippetXml("Console.WriteLine($end$);");

        Assert.Contains("<![CDATA[", xml);
    }

    [Fact]
    public void NoTabstops_EmptyDeclarations()
    {
        var xml = LspSnippetConverter.ToVsSnippetXml("Console.WriteLine(\"hello\");");

        Assert.DoesNotContain("<Declarations>", xml);
    }
}
