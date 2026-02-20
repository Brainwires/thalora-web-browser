namespace ThaloraBrowser.Tests.Helpers;

/// <summary>
/// Collection definition to prevent parallel execution of tests that
/// share the Avalonia headless platform context.
/// </summary>
[CollectionDefinition("Avalonia")]
public class AvaloniaTestCollection : ICollectionFixture<AvaloniaTestFixture>;

public class AvaloniaTestFixture
{
    public AvaloniaTestFixture()
    {
        AvaloniaTestApp.EnsureInitialized();
    }
}
