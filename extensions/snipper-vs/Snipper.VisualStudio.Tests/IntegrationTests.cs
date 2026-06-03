// Layer 4 integration tests — require a live Visual Studio 2022 instance.
//
// These use microsoft/vs-extension-testing (Microsoft.VisualStudio.Extensibility.Testing.Xunit).
// They are guarded by the SNIPPER_VS_INTEGRATION_TESTS environment variable so that the
// regular CI matrix (ubuntu-latest) skips them; the dedicated vsix-test-vs Windows job
// sets that variable and runs them against the pre-installed VS 2022 Community instance.
//
// TODO: add the NuGet package and un-comment once the VSIX itself compiles in CI.
//
//   <PackageReference Include="Microsoft.VisualStudio.Extensibility.Testing.Xunit" Version="0.*" />
//
// Example test shape (requires VS runtime):
//
// [Collection(nameof(SharedIntegrationHostFixture))]
// public sealed class ExtensionActivationTests : AbstractIdeIntegrationTest
// {
//     public ExtensionActivationTests(VisualStudioInstanceFactory factory,
//                                     ITestOutputHelper output)
//         : base(factory, output) { }
//
//     [IdeFact]
//     public async Task PackageLoads()
//     {
//         await TestServices.Shell.WaitForComponentModelAsync(CancellationToken.None);
//         // Assert that SnipperPackage was loaded.
//     }
//
//     [IdeFact]
//     public async Task CommandsRegistered()
//     {
//         var commands = await TestServices.Shell.GetCommandsAsync(CancellationToken.None);
//         Assert.Contains(commands, c => c.Contains("snipper.scaffoldConstructor"));
//     }
// }

// Placeholder so the file compiles without the testing package.
namespace Snipper.VisualStudio.Tests;

// (no types — integration tests are TODO pending VS testing infra)
