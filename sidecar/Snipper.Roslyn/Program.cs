// Snipper.Roslyn — Roslyn sidecar for receiver-type semantic enrichment.
//
// Protocol: JSON-RPC 2.0, one object per line, stdin → stdout.
//
// Supported method:
//   receiverType  { source: string, offset: number }
//               → { types: string[] }
//
// The `types` array contains the fully-qualified name of the receiver's
// concrete type followed by all of its implemented interfaces and base
// types, from most- to least-specific.  An empty array means the type
// could not be resolved (parse error, unknown symbol, etc.).
//
// Crash/exit: the sidecar exits when stdin is closed (parent process died).
// snipper-lsp detects the broken pipe and falls back to CST-only mode.

using System.Text.Json;
using System.Text.Json.Nodes;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

Console.InputEncoding  = System.Text.Encoding.UTF8;
Console.OutputEncoding = System.Text.Encoding.UTF8;

string? line;
while ((line = Console.ReadLine()) != null)
{
    if (string.IsNullOrWhiteSpace(line))
        continue;

    JsonNode? req = null;
    try { req = JsonNode.Parse(line); } catch { }
    if (req is null) continue;

    var id     = req["id"];
    var method = req["method"]?.GetValue<string>();
    var @params = req["params"];

    if (method == "receiverType" && @params is not null)
    {
        var source = @params["source"]?.GetValue<string>() ?? "";
        var offset = @params["offset"]?.GetValue<int>() ?? 0;

        string[] types;
        try   { types = GetReceiverTypes(source, offset); }
        catch { types = []; }

        var result   = new { types };
        var response = new { jsonrpc = "2.0", id, result };
        Console.WriteLine(JsonSerializer.Serialize(response));
    }
    else
    {
        // Unknown method — return an empty result so the client doesn't hang.
        var response = new { jsonrpc = "2.0", id, result = (object?)null };
        Console.WriteLine(JsonSerializer.Serialize(response));
    }
}

static string[] GetReceiverTypes(string source, int offset)
{
    var tree        = CSharpSyntaxTree.ParseText(source);
    var compilation = CSharpCompilation.Create(
        "SnipperAnalysis",
        syntaxTrees: [tree],
        references:  BasicReferences());

    var model  = compilation.GetSemanticModel(tree);
    var root   = tree.GetRoot();
    var token  = root.FindToken(Math.Clamp(offset, 0, source.Length));
    SyntaxNode? node = token.Parent;

    // Walk up until we find a member-access expression.
    while (node is not null)
    {
        if (node is MemberAccessExpressionSyntax mae)
        {
            var typeInfo = model.GetTypeInfo(mae.Expression);
            if (typeInfo.Type is ITypeSymbol t)
                return CollectTypeHierarchy(t).Distinct(StringComparer.Ordinal).ToArray();
            break;
        }
        node = node.Parent;
    }
    return [];
}

static IEnumerable<string> CollectTypeHierarchy(ITypeSymbol type)
{
    yield return type.ToDisplayString();
    foreach (var iface in type.AllInterfaces)
        yield return iface.ToDisplayString();
    if (type.BaseType is { } bt && bt.SpecialType != SpecialType.System_Object)
        foreach (var t in CollectTypeHierarchy(bt))
            yield return t;
}

static MetadataReference[] BasicReferences() =>
[
    MetadataReference.CreateFromFile(typeof(object).Assembly.Location),
    MetadataReference.CreateFromFile(
        typeof(System.Collections.Generic.IEnumerable<>).Assembly.Location),
    MetadataReference.CreateFromFile(
        typeof(System.Linq.Enumerable).Assembly.Location),
];
