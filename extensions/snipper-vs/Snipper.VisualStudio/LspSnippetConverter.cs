using System;
using System.Collections.Generic;
using System.Security;
using System.Text;
using System.Text.RegularExpressions;

namespace Snipper.VisualStudio
{
    internal static class LspSnippetConverter
    {
        // Matches ${1:label} — numbered placeholders with labels
        private static readonly Regex PlaceholderRegex =
            new(@"\$\{(\d+):([^}]+)\}", RegexOptions.Compiled);

        // Matches $0 — final cursor position
        private static readonly Regex FinalCursorRegex =
            new(@"\$0", RegexOptions.Compiled);

        /// <summary>
        /// Converts an LSP snippet body to a VS XML snippet document.
        /// Returns (xmlContent, orderedDeclarations) where xmlContent is the
        /// full snippet XML and orderedDeclarations is the list of field IDs in
        /// tab-order (same order as they appear first in the body).
        /// </summary>
        public static string ToVsSnippetXml(string lspBody, string title = "Snipper")
        {
            // Collect placeholder info in first-occurrence order (tab order).
            var seen = new Dictionary<string, string>(); // id → label
            var ordered = new List<(string id, string label)>();

            var body = PlaceholderRegex.Replace(lspBody, m =>
            {
                var num = m.Groups[1].Value;
                var label = m.Groups[2].Value;
                var id = SanitizeId(label, num);

                if (!seen.ContainsKey(num))
                {
                    seen[num] = id;
                    ordered.Add((id, label));
                }

                return $"${seen[num]}$";
            });

            // Replace $0 with VS $end$
            body = FinalCursorRegex.Replace(body, "$end$");

            var sb = new StringBuilder();
            sb.AppendLine("<?xml version=\"1.0\" encoding=\"utf-8\"?>");
            sb.AppendLine("<CodeSnippets xmlns=\"http://schemas.microsoft.com/VisualStudio/2005/CodeSnippet\">");
            sb.AppendLine("  <CodeSnippet Format=\"1.0.0\">");
            sb.AppendLine("    <Header>");
            sb.AppendLine($"      <Title>{SecurityElement.Escape(title)}</Title>");
            sb.AppendLine("      <Shortcut></Shortcut>");
            sb.AppendLine("    </Header>");
            sb.AppendLine("    <Snippet>");

            if (ordered.Count > 0)
            {
                sb.AppendLine("      <Declarations>");
                foreach (var (id, label) in ordered)
                {
                    sb.AppendLine("        <Literal>");
                    sb.AppendLine($"          <ID>{SecurityElement.Escape(id)}</ID>");
                    sb.AppendLine($"          <Default>{SecurityElement.Escape(label)}</Default>");
                    sb.AppendLine("        </Literal>");
                }
                sb.AppendLine("      </Declarations>");
            }

            sb.AppendLine("      <Code Language=\"CSharp\"><![CDATA[");
            sb.Append(body);
            sb.AppendLine("]]></Code>");
            sb.AppendLine("    </Snippet>");
            sb.AppendLine("  </CodeSnippet>");
            sb.AppendLine("</CodeSnippets>");

            return sb.ToString();
        }

        private static string SanitizeId(string label, string fallbackNum)
        {
            // VS snippet IDs must be valid XML NCNames; strip non-alphanumeric chars.
            var sb = new StringBuilder();
            foreach (var c in label)
            {
                if (char.IsLetterOrDigit(c) || c == '_')
                    sb.Append(c);
            }
            var result = sb.ToString();
            return result.Length > 0 ? result : $"field{fallbackNum}";
        }
    }
}
