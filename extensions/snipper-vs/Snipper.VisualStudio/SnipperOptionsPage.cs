using System.ComponentModel;
using Microsoft.VisualStudio.Shell;

namespace Snipper.VisualStudio
{
    public sealed class SnipperOptionsPage : DialogPage
    {
        [Category("Snipper")]
        [DisplayName("Server Path")]
        [Description("Path to the snipper-lsp.exe binary. Leave empty to use the bundled binary or discover via PATH.")]
        public string ServerPath { get; set; } = string.Empty;

        [Category("Snipper")]
        [DisplayName("Roslyn Sidecar Path")]
        [Description("Path to Snipper.Roslyn.exe sidecar binary. Leave empty to disable C# type-aware filtering.")]
        public string RoslynPath { get; set; } = string.Empty;
    }
}
