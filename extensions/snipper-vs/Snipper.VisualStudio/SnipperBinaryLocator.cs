using System;
using System.IO;
using System.Reflection;

namespace Snipper.VisualStudio
{
    internal static class SnipperBinaryLocator
    {
        private const string BinaryName = "snipper-lsp.exe";

        /// <summary>
        /// Resolution order:
        ///   1. Explicit path from Tools › Options
        ///   2. Bundled binary alongside the extension DLL (bin\snipper-lsp.exe)
        ///   3. snipper-lsp.exe on PATH
        /// Returns null if none found.
        /// </summary>
        public static string? Resolve(string? configuredPath = null)
        {
            if (!string.IsNullOrWhiteSpace(configuredPath) && File.Exists(configuredPath))
                return configuredPath;

            var extensionDir = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location)!;
            var bundled = Path.Combine(extensionDir, "bin", BinaryName);
            if (File.Exists(bundled))
                return bundled;

            // Fall through to PATH
            return FindOnPath(BinaryName);
        }

        private static string? FindOnPath(string fileName)
        {
            var paths = Environment.GetEnvironmentVariable("PATH") ?? string.Empty;
            foreach (var dir in paths.Split(Path.PathSeparator))
            {
                var full = Path.Combine(dir.Trim(), fileName);
                if (File.Exists(full))
                    return full;
            }
            return null;
        }
    }
}
