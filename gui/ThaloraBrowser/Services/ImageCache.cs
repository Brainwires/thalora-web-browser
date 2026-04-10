using System.Collections.Concurrent;
using Avalonia.Media.Imaging;

namespace ThaloraBrowser.Services;

/// <summary>
/// Downloads and caches images for the HTML rendering engine.
/// Thread-safe, with automatic eviction of old entries.
/// </summary>
public sealed class ImageCache : IDisposable
{
    private readonly ConcurrentDictionary<string, CacheEntry> _cache = new();
    private readonly HttpClient _httpClient;
    private readonly int _maxEntries;
    private bool _disposed;

    private record CacheEntry(Bitmap Image, DateTime LastAccessed);

    public ImageCache(int maxEntries = 200)
    {
        _maxEntries = maxEntries;
        _httpClient = new HttpClient();
        _httpClient.DefaultRequestHeaders.UserAgent.ParseAdd(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");
    }

    /// <summary>
    /// Get a cached image or download it. Returns null if download fails.
    /// </summary>
    public async Task<Bitmap?> GetImageAsync(string url, string? baseUrl = null)
    {
        if (_disposed) return null;

        // Handle data: URIs directly — decode base64 and load as bitmap
        if (url.StartsWith("data:", StringComparison.OrdinalIgnoreCase))
        {
            if (_cache.TryGetValue(url, out var dataEntry))
                return dataEntry.Image;
            try
            {
                var comma = url.IndexOf(',');
                if (comma < 0) return null;
                var base64 = url[(comma + 1)..];
                var bytes = Convert.FromBase64String(base64);
                using var ms = new System.IO.MemoryStream(bytes);
                var bmp = new Bitmap(ms);
                _cache[url] = new CacheEntry(bmp, DateTime.UtcNow);
                return bmp;
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine($"[ImageCache] Failed to decode data URI: {ex.Message}");
                return null;
            }
        }

        // Resolve relative URLs
        var absoluteUrl = ResolveUrl(url, baseUrl);
        if (absoluteUrl == null) return null;

        // Check cache first
        if (_cache.TryGetValue(absoluteUrl, out var entry))
        {
            _cache[absoluteUrl] = entry with { LastAccessed = DateTime.UtcNow };
            return entry.Image;
        }

        // Download the image
        try
        {
            var response = await _httpClient.GetAsync(absoluteUrl);
            if (!response.IsSuccessStatusCode)
            {
                Console.Error.WriteLine($"[ImageCache] HTTP {(int)response.StatusCode} for {absoluteUrl}");
                return null;
            }

            var bytes = await response.Content.ReadAsByteArrayAsync();
            var bitmap = LoadBitmapFromBytes(bytes);
            if (bitmap == null)
            {
                Console.Error.WriteLine($"[ImageCache] Failed to decode image '{absoluteUrl}'");
                return null;
            }

            // Evict old entries if at capacity
            if (_cache.Count >= _maxEntries)
                EvictOldest();

            _cache[absoluteUrl] = new CacheEntry(bitmap, DateTime.UtcNow);
            return bitmap;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[ImageCache] Failed to load '{absoluteUrl}': {ex.Message}");
            return null;
        }
    }

    /// <summary>
    /// Check if an image is already cached.
    /// </summary>
    public bool IsCached(string url, string? baseUrl = null)
    {
        var absoluteUrl = ResolveUrl(url, baseUrl);
        return absoluteUrl != null && _cache.ContainsKey(absoluteUrl);
    }

    /// <summary>
    /// Get a cached bitmap synchronously. Returns null if not cached.
    /// </summary>
    public Bitmap? GetCachedBitmap(string url, string? baseUrl = null)
    {
        var absoluteUrl = ResolveUrl(url, baseUrl);
        if (absoluteUrl != null && _cache.TryGetValue(absoluteUrl, out var entry))
        {
            _cache[absoluteUrl] = entry with { LastAccessed = DateTime.UtcNow };
            return entry.Image;
        }
        return null;
    }

    /// <summary>
    /// Clear the entire cache.
    /// </summary>
    public void Clear()
    {
        foreach (var entry in _cache.Values)
            entry.Image.Dispose();
        _cache.Clear();
    }

    internal static string? ResolveUrl(string url, string? baseUrl)
    {
        // Protocol-relative URLs (//host/path) must be handled FIRST.
        // On macOS/Linux, Uri.TryCreate("//host/path", UriKind.Absolute, ...) can succeed
        // with scheme="file" (UNC path interpretation), producing "file://host/path".
        // We must intercept these before the general absolute-URI check.
        if (url.StartsWith("//", StringComparison.Ordinal))
        {
            string scheme = "https"; // default
            if (baseUrl != null && Uri.TryCreate(baseUrl, UriKind.Absolute, out var bu)
                && (bu.Scheme == "http" || bu.Scheme == "https"))
                scheme = bu.Scheme;
            var fullUrl = scheme + ":" + url;
            if (Uri.TryCreate(fullUrl, UriKind.Absolute, out var schemeResolved))
                return schemeResolved.ToString();
        }

        if (Uri.TryCreate(url, UriKind.Absolute, out var absolute)
            && (absolute.Scheme == "http" || absolute.Scheme == "https" || absolute.Scheme == "data"))
            return absolute.ToString();

        if (baseUrl != null && Uri.TryCreate(baseUrl, UriKind.Absolute, out var baseUri))
        {
            if (Uri.TryCreate(baseUri, url, out var resolved)
                && (resolved.Scheme == "http" || resolved.Scheme == "https"))
                return resolved.ToString();
        }

        return null;
    }

    /// <summary>
    /// Decode image bytes to an Avalonia Bitmap using SkiaSharp as a fallback decoder.
    /// SkiaSharp handles formats Avalonia can't (e.g. 8-bit gray+alpha PNGs from Wikimedia).
    /// </summary>
    private static Bitmap? LoadBitmapFromBytes(byte[] bytes)
    {
        // First try Avalonia's native decoder (fast, handles most common formats)
        try
        {
            using var ms = new System.IO.MemoryStream(bytes);
            return new Bitmap(ms);
        }
        catch
        {
            // Fall through to SkiaSharp
        }

        // SkiaSharp handles edge cases: gray+alpha, 16-bit, CMYK, etc.
        try
        {
            using var skBitmap = SkiaSharp.SKBitmap.Decode(bytes);
            if (skBitmap == null) return null;

            // Convert to RGBA8888 so Avalonia can display it
            using var converted = skBitmap.Copy(SkiaSharp.SKColorType.Rgba8888);
            using var skImage = SkiaSharp.SKImage.FromBitmap(converted);
            using var encoded = skImage.Encode(SkiaSharp.SKEncodedImageFormat.Png, 100);
            using var pngStream = new System.IO.MemoryStream(encoded.ToArray());
            return new Bitmap(pngStream);
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[ImageCache] SkiaSharp decode failed: {ex.Message}");
            return null;
        }
    }

    private void EvictOldest()
    {
        var oldest = _cache
            .OrderBy(kvp => kvp.Value.LastAccessed)
            .Take(_cache.Count / 4)
            .Select(kvp => kvp.Key)
            .ToList();

        foreach (var key in oldest)
        {
            if (_cache.TryRemove(key, out var entry))
                entry.Image.Dispose();
        }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        Clear();
        _httpClient.Dispose();
    }
}
