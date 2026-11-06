import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { Apple, Calendar, Download, ExternalLink, Monitor, Terminal } from "lucide-react";
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

interface PlatformFile {
  title: string;
  description: string;
  asset: GitHubAsset;
}

interface Platform {
  key: string;
  name: string;
  icon: ReactNode;
  primary: PlatformFile;
  others: PlatformFile[];
}

const findAsset = (assets: GitHubAsset[], ...extensions: string[]) =>
  assets.find((asset) =>
    extensions.some((extension) =>
      asset.name.toLowerCase().endsWith(extension.toLowerCase()),
    ),
  );

type WindowsArch = "x64" | "x86" | "arm64";
type WindowsKind = "installer" | "portable";

const windowsArch = (name: string): WindowsArch => {
  if (/arm64|aarch64/i.test(name)) return "arm64";
  if (/x86|i686|win32|32-bit/i.test(name)) return "x86";
  return "x64";
};

const windowsKind = (name: string): WindowsKind =>
  /\.zip$/i.test(name) ? "portable" : "installer";

/** Installer/portable pair per architecture, in the order shown on the page. */
const windowsEntries: Array<{
  title: string;
  description: string;
  arch: WindowsArch;
  kind: WindowsKind;
}> = [
  {
    title: "Installer (64-bit)",
    description: "Most Windows 10 and 11 PCs.",
    arch: "x64",
    kind: "installer",
  },
  {
    title: "Portable (64-bit)",
    description: "Unzip and run, no install.",
    arch: "x64",
    kind: "portable",
  },
  {
    title: "Installer (32-bit)",
    description: "Older 32-bit Windows.",
    arch: "x86",
    kind: "installer",
  },
  {
    title: "Portable (32-bit)",
    description: "Unzip and run on 32-bit Windows.",
    arch: "x86",
    kind: "portable",
  },
  {
    title: "Installer (ARM64)",
    description: "Windows 11 on ARM laptops.",
    arch: "arm64",
    kind: "installer",
  },
  {
    title: "Portable (ARM64)",
    description: "Unzip and run on Windows on ARM.",
    arch: "arm64",
    kind: "portable",
  },
];

/**
 * Maps the release assets onto the three platforms we ship, picking a single
 * primary download per platform and listing the remaining formats underneath.
 */
const buildPlatforms = (release: GitHubRelease): Platform[] => {
  const assets = release.assets ?? [];

  const appimage = findAsset(assets, ".AppImage");
  const debian = findAsset(assets, ".deb");
  const rpm = findAsset(assets, ".rpm");
  const tarball = findAsset(assets, ".tar.gz");
  const macos = findAsset(assets, ".dmg", ".app");

  const platforms: Platform[] = [];

  // Windows ships an installer and a portable zip for each architecture.
  const windowsAssets = assets.filter((asset) =>
    /\.(exe|msi|zip)$/i.test(asset.name),
  );

  const windowsFiles: PlatformFile[] = windowsEntries
    .map((entry) => {
      const asset = windowsAssets.find(
        (candidate) =>
          windowsArch(candidate.name) === entry.arch &&
          windowsKind(candidate.name) === entry.kind,
      );
      return asset
        ? { title: entry.title, description: entry.description, asset }
        : null;
    })
    .filter((file): file is PlatformFile => file !== null);

  if (windowsFiles.length > 0) {
    const [primary, ...others] = windowsFiles;
    platforms.push({
      key: "windows",
      name: "Windows",
      icon: <Monitor className="h-5 w-5" />,
      primary,
      others,
    });
  }

  // AppImage is the most portable option, so it is the primary Linux download.
  const linuxFiles: PlatformFile[] = [];
  if (appimage) {
    linuxFiles.push({
      title: "AppImage (x86_64)",
      description: "Any distro. Mark it executable and run.",
      asset: appimage,
    });
  }
  if (debian) {
    linuxFiles.push({
      title: "Debian package (x86_64)",
      description: "Debian, Ubuntu and derivatives.",
      asset: debian,
    });
  }
  if (rpm) {
    linuxFiles.push({
      title: "RPM package (x86_64)",
      description: "Fedora, RHEL and openSUSE.",
      asset: rpm,
    });
  }
  if (tarball) {
    linuxFiles.push({
      title: "Portable tarball (x86_64)",
      description: "Plain binaries to run anywhere.",
      asset: tarball,
    });
  }

  if (linuxFiles.length > 0) {
    const [primary, ...others] = linuxFiles;
    platforms.push({
      key: "linux",
      name: "Linux",
      icon: <Terminal className="h-5 w-5" />,
      primary,
      others,
    });
  }

  if (macos) {
    platforms.push({
      key: "macos",
      name: "macOS",
      icon: <Apple className="h-5 w-5" />,
      primary: {
        title: "DMG installer",
        description: "Apple Silicon and Intel Macs.",
        asset: macos,
      },
      others: [],
    });
  }

  // Fallback so an unexpected asset list still renders something downloadable.
  if (platforms.length === 0 && assets.length > 0) {
    platforms.push({
      key: "other",
      name: release.name || "All platforms",
      icon: <Download className="h-5 w-5" />,
      primary: {
        title: assets[0].name,
        description: "Latest release asset.",
        asset: assets[0],
      },
      others: [],
    });
  }

  return platforms;
};

function PlatformColumn({ platform }: { platform: Platform }) {
  return (
    <td className="block border-b border-slate-800 p-6 align-top last:border-b-0 lg:table-cell lg:border-b-0 lg:border-r lg:border-slate-800 lg:last:border-r-0">
      <div className="flex items-center gap-4">
        <span className="flex h-11 w-11 shrink-0 items-center justify-center border border-slate-700 text-white">
          {platform.icon}
        </span>
        <h3 className="text-2xl font-bold text-white">{platform.name}</h3>
      </div>

      <p className="mt-6 text-white">{platform.primary.title}</p>
      <p className="mt-1 text-sm text-slate-400">
        {platform.primary.description}
      </p>

      <a
        href={platform.primary.asset.browser_download_url}
        target="_blank"
        rel="noopener noreferrer"
        className="mt-5 flex w-full items-center justify-center gap-2 border border-slate-600 px-4 py-3 text-[11px] font-semibold uppercase tracking-[0.18em] text-white transition-colors hover:border-white hover:bg-slate-800/60"
      >
        <Download className="h-4 w-4" />
        Download for {platform.name}
      </a>

      {platform.others.length > 0 && (
        <div className="mt-8">
          <p className="text-[11px] uppercase leading-5 tracking-[0.2em] text-slate-500">
            Other nixobdo files for {platform.name}
          </p>

          <div className="mt-3 divide-y divide-slate-800 border-t border-slate-800">
            {platform.others.map((file) => (
              <a
                key={file.asset.name}
                href={file.asset.browser_download_url}
                target="_blank"
                rel="noopener noreferrer"
                className="group flex items-start justify-between gap-3 py-4"
              >
                <div className="min-w-0">
                  <p className="text-white transition-colors group-hover:text-indigo-400">
                    {file.title}
                  </p>
                  <p className="mt-1 text-sm text-slate-400">
                    {file.description}
                  </p>
                  <p className="mt-2 truncate font-mono text-[11px] text-slate-600">
                    {file.asset.name}
                  </p>
                </div>
                <Download className="mt-1 h-4 w-4 shrink-0 text-slate-500 transition-colors group-hover:text-white" />
              </a>
            ))}
          </div>
        </div>
      )}
    </td>
  );
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
      <div className="max-w-6xl animate-in fade-in duration-500">
        <div className="animate-pulse flex flex-col gap-4">
          <div className="h-10 bg-white/5 rounded w-1/3"></div>
          <div className="h-6 bg-white/5 rounded w-1/2 mb-8"></div>
          <div className="h-24 bg-white/5 rounded-xl w-full"></div>
        </div>
      </div>
    );
  }

  const platforms = latestStable ? buildPlatforms(latestStable) : [];
  const totalDownloads = latestStable
    ? latestStable.assets.reduce((acc, asset) => acc + asset.download_count, 0)
    : 0;

  return (
    <div className="max-w-6xl animate-in fade-in duration-500">
      <h1 className="text-3xl font-bold text-white mb-6">Download</h1>

      {!latestStable ? (
        <div className="bg-slate-900 border border-slate-800 rounded-2xl p-8 text-slate-400 mb-12">
          No stable releases found. Check back later!
        </div>
      ) : (
        <div className="mb-8">
          <h2 className="text-3xl font-extrabold text-white mb-4">
            {latestStable.tag_name}
          </h2>

          <div className="flex flex-wrap items-center gap-3 text-slate-400 text-sm mb-8">
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

          {platforms.length > 0 && (
            <div className="mb-12 overflow-hidden border border-slate-800">
              <table className="block w-full border-collapse text-left lg:table lg:table-fixed">
                <thead className="hidden lg:table-header-group">
                  <tr>
                    {platforms.map((platform, index) => (
                      <th
                        key={platform.key}
                        scope="col"
                        className="border-b border-slate-800 px-6 py-3 font-medium lg:border-r lg:border-slate-800 lg:last:border-r-0"
                      >
                        <div className="flex items-center justify-between gap-4 text-[11px] uppercase tracking-[0.22em] text-slate-500">
                          <span>Platform</span>
                          <span className="tabular-nums">
                            {String(index + 1).padStart(2, "0")}
                          </span>
                        </div>
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody className="block lg:table-row-group">
                  <tr className="block lg:table-row">
                    {platforms.map((platform) => (
                      <PlatformColumn key={platform.key} platform={platform} />
                    ))}
                  </tr>
                </tbody>
              </table>
            </div>
          )}
        </div>
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
