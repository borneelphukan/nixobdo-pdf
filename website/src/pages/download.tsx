import { useEffect, useState } from "react";
import { Calendar, Download, ExternalLink } from "lucide-react";
import { Link } from "react-router-dom";
import GitHubIcon from "@mui/icons-material/GitHub";

interface GitHubAsset {
  name: string;
  browser_download_url: string;
  download_count: number;
  size: number;
}

interface GitHubRelease {
  id: number;
  name: string;
  tag_name: string;
  prerelease: boolean;
  published_at: string;
  html_url: string;
  assets: GitHubAsset[];
}

export function DocsDownload() {
  const [latestStable, setLatestStable] = useState<GitHubRelease | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchReleases() {
      try {
        const response = await fetch(
          "https://api.github.com/repos/borneelphukan/nixobdo-pdf/releases",
        );
        if (!response.ok) throw new Error("Failed to fetch releases");
        const data: GitHubRelease[] = await response.json();

        const stable = data.find((r) => !r.prerelease);
        if (stable) setLatestStable(stable);
      } catch (error) {
        console.error("Error fetching releases:", error);
      } finally {
        setLoading(false);
      }
    }

    fetchReleases();
  }, []);

  const getDownloadAssets = (release: GitHubRelease): GitHubAsset[] => {
    if (!release.assets || release.assets.length === 0) return [];

    const windows = release.assets.find(
      (a) => a.name.endsWith(".exe") || a.name.endsWith(".msi"),
    );
    const debian = release.assets.find((a) => a.name.endsWith(".deb"));
    const tarball = release.assets.find(
      (a) => a.name.endsWith(".tar.gz"),
    );
    const appimage = release.assets.find((a) => a.name.endsWith(".AppImage"));
    const macos = release.assets.find(
      (a) => a.name.endsWith(".dmg") || a.name.endsWith(".app"),
    );

    if (!windows && !debian && !tarball && !appimage && !macos) {
      return [release.assets[0]];
    }

    return [windows, debian, tarball, appimage, macos].filter(
      (a): a is GitHubAsset => a !== undefined && a !== null,
    );
  };

  const formatDate = (dateString: string) => {
    const options: Intl.DateTimeFormatOptions = {
      year: "numeric",
      month: "numeric",
      day: "numeric",
    };
    return new Date(dateString).toLocaleDateString(undefined, options);
  };

  if (loading) {
    return (
      <div className="max-w-4xl animate-in fade-in duration-500">
        <div className="animate-pulse flex flex-col gap-4">
          <div className="h-10 bg-white/5 rounded w-1/3"></div>
          <div className="h-6 bg-white/5 rounded w-1/2 mb-8"></div>
          <div className="h-24 bg-white/5 rounded-xl w-full"></div>
        </div>
      </div>
    );
  }

  const downloadAssets = latestStable ? getDownloadAssets(latestStable) : [];
  const totalDownloads = latestStable
    ? latestStable.assets.reduce((acc, asset) => acc + asset.download_count, 0)
    : 0;

  return (
    <div className="max-w-4xl animate-in fade-in duration-500">
      <h1 className="text-3xl font-bold text-white mb-6">Download</h1>

      {!latestStable ? (
        <div className="bg-slate-900 border border-slate-800 rounded-2xl p-8 text-slate-400 mb-12">
          No stable releases found. Check back later!
        </div>
      ) : (
        <>
          <div className="mb-8">
            <h2 className="text-3xl font-extrabold text-white mb-4">
              {latestStable.tag_name}
            </h2>

            <div className="flex flex-wrap items-center gap-3 text-slate-400 text-sm">
              <div className="flex items-center gap-1.5">
                <Calendar className="w-4 h-4" />
                {formatDate(latestStable.published_at)}
              </div>
              <span className="text-slate-600">•</span>
              <div className="flex items-center gap-1.5">
                <Download className="w-4 h-4" />
                {totalDownloads}
              </div>
              <span className="text-slate-600">•</span>
              <a
                href={latestStable.html_url}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-1.5 hover:text-white transition-colors"
              >
                <GitHubIcon className="w-4 h-4" />
                GitHub
                <ExternalLink className="w-3 h-3 ml-0.5" />
              </a>
            </div>

            {downloadAssets.length > 0 && (
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 mb-12">
                {/* Windows Section */}
                <div>
                  {downloadAssets.find((a) => a.name.endsWith(".exe") || a.name.endsWith(".msi")) && (
                    <div className="bg-slate-950/50 border-purple-500/60 rounded-xl p-6">
                      <div className="flex items-center gap-3 mb-4">
                        <div className="bg-slate-800/80 p-2 rounded-lg text-purple-400">
                          <Download className="w-5 h-5" />
                        </div>
                        <div>
                          <h3 className="text-white font-semibold">Windows</h3>
                          <p className="text-slate-400 text-sm">Windows installer</p>
                        </div>
                      </div>
                      <a
                        href={downloadAssets.find((a) => a.name.endsWith(".exe") || a.name.endsWith(".msi"))!.browser_download_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="w-full bg-purple-600/60 text-purple-400 rounded-lg py-3 px-4 text-sm font-medium hover:bg-purple-500/70 transition-colors"
                      >
                        Download for Windows
                      </a>
                    </div>
                  )}
                </div>

                {/* Linux Section */}
                <div>
                  <div className="flex items-center justify-between mb-4">
                    <h3 className="text-white font-medium">Linux</h3>
                    <span className="text-slate-500 text-xs">Package options</span>
                  </div>
                  <div className="grid grid-cols-2 gap-3">
                    {/* Debian Package */}
                    {downloadAssets.find((a) => a.name.endsWith(".deb")) && (
                      <div>
                        <div className="bg-slate-800/80 p-3 rounded-lg text-orange-400">
                          <Download className="w-4 h-4" />
                        </div>
                        <div>
                          <h4 className="text-white font-medium text-sm">Debian package (x86_64)</h4>
                          <p className="text-slate-500 text-xs mt-1">Debian, Ubuntu and derivatives</p>
                        </div>
                      </div>
                    )}
                    {/* Tarball */}
                    {downloadAssets.find((a) => a.name.endsWith(".tar.gz")) && (
                      <div>
                        <div className="bg-slate-800/80 p-3 rounded-lg text-green-400">
                          <Download className="w-4 h-4" />
                        </div>
                        <div>
                          <h4 className="text-white font-medium text-sm">x86_64</h4>
                          <p className="text-slate-500 text-xs mt-1">Plain binaries to run anywhere</p>
                        </div>
                      </div>
                    )}
                    {/* AppImage */}
                    {downloadAssets.find((a) => a.name.endsWith(".AppImage")) && (
                      <div>
                        <div className="bg-slate-800/80 p-3 rounded-lg text-green-400">
                          <Download className="w-4 h-4" />
                        </div>
                        <div>
                          <h4 className="text-white font-medium text-sm">AppImage (x86_64)</h4>
                          <p className="text-slate-500 text-xs mt-1">Run as portable application on most Linux distributions</p>
                        </div>
                      </div>
                    )}
                  </div>
                </div>

                {/* macOS Section */}
                {downloadAssets.find((a) => a.name.endsWith(".dmg") || a.name.endsWith(".app")) && (
                  <div>
                    <div className="bg-slate-950/50 border-orange-500/60 rounded-xl p-6">
                      <div className="flex items-center gap-3 mb-4">
                        <div className="bg-slate-800/80 p-2 rounded-lg text-orange-400">
                          <Download className="w-5 h-5" />
                        </div>
                        <div>
                          <h3 className="text-white font-semibold">macOS</h3>
                          <p className="text-slate-400 text-sm">DMG installer</p>
                        </div>
                      </div>
                      <a
                        href={downloadAssets.find((a) => a.name.endsWith(".dmg") || a.name.endsWith(".app"))!.browser_download_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="w-full bg-orange-600/60 text-orange-400 rounded-lg py-3 px-4 text-sm font-medium hover:bg-orange-500/70 transition-colors"
                      >
                        Download for macOS
                      </a>
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>
        </>
      )}

      <h2 className="text-2xl font-bold text-white mb-4 mt-8">
        Recent Changes
      </h2>
      <p className="text-slate-400 mb-6">
        Check out the latest improvements and bug fixes.
      </p>

      <Link
        to="/docs/changelog"
        className="inline-flex items-center text-indigo-400 hover:text-indigo-300 font-medium transition group"
      >
        View Full Changelog
        <span className="ml-1 transition-transform group-hover:translate-x-1">
          →
        </span>
      </Link>
    </div>
  );
}