using Microsoft.VisualStudio.Sdk.TestFramework;
using Xunit;

// Defines the "MockedVS" xunit collection that all integration test classes belong to.
// The GlobalServiceProvider fixture provides a mocked VS service container; tests in
// this collection run on a simulated VS UI thread without a real IDE instance.
[CollectionDefinition(nameof(MockedVS))]
public class MockedVS : ICollectionFixture<GlobalServiceProvider>
{
}
