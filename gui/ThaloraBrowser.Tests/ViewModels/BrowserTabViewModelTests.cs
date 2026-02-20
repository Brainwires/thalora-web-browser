using ThaloraBrowser.ViewModels;
using ThaloraBrowser.Services;

namespace ThaloraBrowser.Tests.ViewModels;

public class BrowserTabViewModelTests
{
    // ---------------------------------------------------------------
    // Default state
    // ---------------------------------------------------------------

    [Fact]
    public void Constructor_DefaultTitle_IsNewTab()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        var vm = new BrowserTabViewModel(engine);

        vm.Title.Should().Be("New Tab");
    }

    [Fact]
    public void Constructor_DefaultState_UrlEmptyIsLoadingFalseStatusReady()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        var vm = new BrowserTabViewModel(engine);

        vm.Url.Should().BeEmpty();
        vm.IsLoading.Should().BeFalse();
        vm.StatusText.Should().Be("Ready");
    }

    [Fact]
    public void Constructor_TwoInstances_HaveDifferentIds()
    {
        var engine1 = Substitute.For<IThaloraBrowserEngine>();
        var engine2 = Substitute.For<IThaloraBrowserEngine>();
        var vm1 = new BrowserTabViewModel(engine1);
        var vm2 = new BrowserTabViewModel(engine2);

        vm1.Id.Should().NotBe(vm2.Id);
    }

    // ---------------------------------------------------------------
    // NavigateAsync
    // ---------------------------------------------------------------

    [Fact]
    public async Task NavigateAsync_UrlWithoutScheme_PrependsHttps()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.NavigateAsync(Arg.Any<string>()).Returns("<html>ok</html>");
        engine.GetCurrentUrlAsync().Returns("https://example.com");
        engine.GetPageTitleAsync().Returns("Example");
        var vm = new BrowserTabViewModel(engine);

        await vm.NavigateAsync("example.com");

        await engine.Received(1).NavigateAsync("https://example.com");
    }

    [Fact]
    public async Task NavigateAsync_UrlWithHttpScheme_StaysAsIs()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.NavigateAsync(Arg.Any<string>()).Returns("<html>ok</html>");
        engine.GetCurrentUrlAsync().Returns("http://example.com");
        engine.GetPageTitleAsync().Returns("Example");
        var vm = new BrowserTabViewModel(engine);

        await vm.NavigateAsync("http://example.com");

        await engine.Received(1).NavigateAsync("http://example.com");
    }

    [Fact]
    public async Task NavigateAsync_IsLoadingFalseAfterCompletion()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.NavigateAsync(Arg.Any<string>()).Returns("<html>ok</html>");
        engine.GetCurrentUrlAsync().Returns("https://example.com");
        engine.GetPageTitleAsync().Returns("Example");
        var vm = new BrowserTabViewModel(engine);

        await vm.NavigateAsync("https://example.com");

        vm.IsLoading.Should().BeFalse();
    }

    [Fact]
    public async Task NavigateAsync_SetsHtmlContentFromEngine()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.NavigateAsync(Arg.Any<string>()).Returns("<html><body>Hello</body></html>");
        engine.GetCurrentUrlAsync().Returns("https://example.com");
        engine.GetPageTitleAsync().Returns("Hello Page");
        var vm = new BrowserTabViewModel(engine);

        await vm.NavigateAsync("https://example.com");

        vm.HtmlContent.Should().Be("<html><body>Hello</body></html>");
    }

    [Fact]
    public async Task NavigateAsync_UrlUpdatedFromGetCurrentUrl_SimulatingRedirect()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.NavigateAsync(Arg.Any<string>()).Returns("<html>redirected</html>");
        engine.GetCurrentUrlAsync().Returns("https://example.com/redirected");
        engine.GetPageTitleAsync().Returns("Redirected");
        var vm = new BrowserTabViewModel(engine);

        await vm.NavigateAsync("https://example.com/original");

        vm.Url.Should().Be("https://example.com/redirected");
    }

    [Fact]
    public async Task NavigateAsync_TitleSetFromGetPageTitle()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.NavigateAsync(Arg.Any<string>()).Returns("<html>test</html>");
        engine.GetCurrentUrlAsync().Returns("https://example.com");
        engine.GetPageTitleAsync().Returns("My Page Title");
        var vm = new BrowserTabViewModel(engine);

        await vm.NavigateAsync("https://example.com");

        vm.Title.Should().Be("My Page Title");
    }

    [Fact]
    public async Task NavigateAsync_NullTitle_FallsBackToTruncateUrl()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.NavigateAsync(Arg.Any<string>()).Returns("<html>test</html>");
        engine.GetCurrentUrlAsync().Returns("https://example.com/short");
        engine.GetPageTitleAsync().Returns((string?)null);
        var vm = new BrowserTabViewModel(engine);

        await vm.NavigateAsync("https://example.com/short");

        vm.Title.Should().Be(BrowserTabViewModel.TruncateUrl("https://example.com/short"));
    }

    [Fact]
    public async Task NavigateAsync_EngineThrows_SetsErrorStatusAndHtmlContent()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.NavigateAsync(Arg.Any<string>())
            .Returns<string?>(_ => throw new InvalidOperationException("Connection failed"));
        var vm = new BrowserTabViewModel(engine);

        await vm.NavigateAsync("https://bad-site.example");

        vm.StatusText.Should().Contain("Error:");
        vm.StatusText.Should().Contain("Connection failed");
        vm.HtmlContent.Should().Contain("Error");
        vm.HtmlContent.Should().Contain("Connection failed");
        vm.IsLoading.Should().BeFalse();
    }

    // ---------------------------------------------------------------
    // GoBackAsync
    // ---------------------------------------------------------------

    [Fact]
    public async Task GoBackAsync_Success_RefreshesState()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.GoBackAsync().Returns(true);
        engine.GetPageHtmlAsync().Returns("<html>back</html>");
        engine.GetCurrentUrlAsync().Returns("https://example.com/previous");
        engine.GetPageTitleAsync().Returns("Previous Page");
        var vm = new BrowserTabViewModel(engine);

        var result = await vm.GoBackAsync();

        result.Should().BeTrue();
        vm.HtmlContent.Should().Be("<html>back</html>");
        vm.Url.Should().Be("https://example.com/previous");
        vm.Title.Should().Be("Previous Page");
    }

    [Fact]
    public async Task GoBackAsync_Failure_DoesNotRefreshState()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.GoBackAsync().Returns(false);
        var vm = new BrowserTabViewModel(engine);
        var originalTitle = vm.Title;

        var result = await vm.GoBackAsync();

        result.Should().BeFalse();
        vm.Title.Should().Be(originalTitle);
        await engine.DidNotReceive().GetPageHtmlAsync();
    }

    [Fact]
    public async Task GoBackAsync_IsLoadingFalseAfterCompletion()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.GoBackAsync().Returns(true);
        engine.GetPageHtmlAsync().Returns("<html>back</html>");
        engine.GetCurrentUrlAsync().Returns("https://example.com");
        engine.GetPageTitleAsync().Returns("Title");
        var vm = new BrowserTabViewModel(engine);

        await vm.GoBackAsync();

        vm.IsLoading.Should().BeFalse();
    }

    // ---------------------------------------------------------------
    // GoForwardAsync
    // ---------------------------------------------------------------

    [Fact]
    public async Task GoForwardAsync_Success_RefreshesState()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.GoForwardAsync().Returns(true);
        engine.GetPageHtmlAsync().Returns("<html>forward</html>");
        engine.GetCurrentUrlAsync().Returns("https://example.com/next");
        engine.GetPageTitleAsync().Returns("Next Page");
        var vm = new BrowserTabViewModel(engine);

        var result = await vm.GoForwardAsync();

        result.Should().BeTrue();
        vm.HtmlContent.Should().Be("<html>forward</html>");
        vm.Url.Should().Be("https://example.com/next");
    }

    [Fact]
    public async Task GoForwardAsync_Failure_DoesNotRefreshState()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.GoForwardAsync().Returns(false);
        var vm = new BrowserTabViewModel(engine);

        var result = await vm.GoForwardAsync();

        result.Should().BeFalse();
        await engine.DidNotReceive().GetPageHtmlAsync();
    }

    // ---------------------------------------------------------------
    // ReloadAsync
    // ---------------------------------------------------------------

    [Fact]
    public async Task ReloadAsync_UpdatesContentAndTitle()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.NavigateAsync(Arg.Any<string>()).Returns("<html>initial</html>");
        engine.GetCurrentUrlAsync().Returns("https://example.com");
        engine.GetPageTitleAsync().Returns("Initial");
        var vm = new BrowserTabViewModel(engine);

        // Navigate first to set URL
        await vm.NavigateAsync("https://example.com");

        engine.ReloadAsync().Returns("<html>reloaded</html>");
        engine.GetPageTitleAsync().Returns("Reloaded Page");

        await vm.ReloadAsync();

        vm.HtmlContent.Should().Be("<html>reloaded</html>");
        vm.Title.Should().Be("Reloaded Page");
        vm.IsLoading.Should().BeFalse();
    }

    [Fact]
    public async Task ReloadAsync_EngineThrows_SetsErrorStatus()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        engine.ReloadAsync()
            .Returns<string?>(_ => throw new InvalidOperationException("Reload failed"));
        var vm = new BrowserTabViewModel(engine);

        await vm.ReloadAsync();

        vm.StatusText.Should().Contain("Error:");
        vm.StatusText.Should().Contain("Reload failed");
        vm.IsLoading.Should().BeFalse();
    }

    // ---------------------------------------------------------------
    // TruncateUrl
    // ---------------------------------------------------------------

    [Fact]
    public void TruncateUrl_ShortUrl_ReturnsUnchanged()
    {
        var url = "https://example.com"; // 19 chars, well under 40

        var result = BrowserTabViewModel.TruncateUrl(url);

        result.Should().Be(url);
    }

    [Fact]
    public void TruncateUrl_LongUrl_ReturnsFirst37CharsWithEllipsis()
    {
        var url = "https://example.com/very/long/path/that/exceeds/forty/characters";

        var result = BrowserTabViewModel.TruncateUrl(url);

        result.Should().HaveLength(40);
        result.Should().EndWith("...");
        result.Should().Be(url[..37] + "...");
    }

    // ---------------------------------------------------------------
    // Dispose
    // ---------------------------------------------------------------

    [Fact]
    public void Dispose_CallsEngineDispose()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        var vm = new BrowserTabViewModel(engine);

        vm.Dispose();

        engine.Received(1).Dispose();
    }

    [Fact]
    public async Task Dispose_NavigateAfterDispose_IsNoOp()
    {
        var engine = Substitute.For<IThaloraBrowserEngine>();
        var vm = new BrowserTabViewModel(engine);

        vm.Dispose();
        await vm.NavigateAsync("https://example.com");

        await engine.DidNotReceive().NavigateAsync(Arg.Any<string>());
    }
}
