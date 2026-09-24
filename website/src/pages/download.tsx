import { useEffect, useState } from 'react';
import { Calendar, Download, ChevronRight, ExternalLink } from 'lucide-react';
import { Link } from 'react-router-dom';
import GitHubIcon from '@mui/icons-material/GitHub';

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
  const [nightlies, setNightlies] = useState<GitHubRelease[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchReleases() {
      try {
        const response = await fetch('https://api.github.com/repos/borneelphukan/nixobdo-pdf/releases');
        if (!response.ok) throw new Error('Failed to fetch releases');
        const data: GitHubRelease[] = await response.json();
        
        const stable = data.find(r => !r.prerelease);
        if (stable) setLatestStable(stable);
        
        const prereleases = data.filter(r => r.prerelease).slice(0, 3);
        setNightlies(prereleases);
      } catch (error) {
        console.error('Error fetching releases:', error);
      } finally {
        setLoading(false);
      }
    }
    
    fetchReleases();
  }, []);

  const getDownloadAssets = (release: GitHubRelease): GitHubAsset[] => {
    if (!release.assets || release.assets.length === 0) return [];
    
    const windows = release.assets.find(a => a.name.endsWith('.exe') || a.name.endsWith('.msi'));
    const ubuntuDeb = release.assets.find(a => a.name.endsWith('.deb'));
    const ubuntuTar = release.assets.find(a => a.name.endsWith('.tar.gz'));
    
    if (!windows && !ubuntuDeb && !ubuntuTar) {
      return [release.assets[0]];
    }
    
    return [windows, ubuntuDeb, ubuntuTar].filter((a): a is GitHubAsset => a !== undefined && a !== null);
  };

  const formatDate = (dateString: string) => {
    const options: Intl.DateTimeFormatOptions = { year: 'numeric', month: 'numeric', day: 'numeric' };
    return new Date(dateString).toLocaleDateString(undefined, options);
  };

  const formatSize = (bytes: number) => {
    return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
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
  const totalDownloads = latestStable ? latestStable.assets.reduce((acc, asset) => acc + asset.download_count, 0) : 0;

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
            <h2 className="text-3xl font-extrabold text-white mb-4">{latestStable.tag_name}</h2>
            
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
          </div>

          {downloadAssets.length > 0 && (
            <div className="grid gap-4 mb-12">
              {downloadAssets.map(asset => {
                let platformIconColor = "text-purple-400";
                let platformBorderColor = "border-purple-400/60";
                let platformHoverChevron = "text-purple-500";
                let platformTitle = "Windows";
                
                if (asset.name.endsWith('.deb')) {
                  platformIconColor = "text-orange-400";
                  platformBorderColor = "border-orange-400/60";
                  platformHoverChevron = "text-orange-500";
                  platformTitle = "Ubuntu / Debian";
                } else if (asset.name.endsWith('.tar.gz')) {
                  platformIconColor = "text-amber-400";
                  platformBorderColor = "border-amber-400/60";
                  platformHoverChevron = "text-amber-500";
                  platformTitle = "Linux (Portable)";
                }

                return (
                  <a 
                    key={asset.name}
                    href={asset.browser_download_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className={`group block bg-slate-950/50 border ${platformBorderColor} rounded-xl p-4 transition-all hover:bg-slate-900/80`}
                  >
                    <div className="flex items-center justify-between gap-4">
                      <div className="flex items-center gap-4">
                        <div className={`bg-slate-800/80 p-3 rounded-lg ${platformIconColor}`}>
                          <Download className="w-6 h-6" />
                        </div>
                        <div>
                          <h3 className="text-white font-semibold text-lg">{platformTitle}</h3>
                          <p className="text-slate-500 text-sm mt-0.5">
                            {asset.name} • {formatSize(asset.size)} • {asset.download_count} downloads
                          </p>
                        </div>
                      </div>
                      <div className="pr-2">
                        <ChevronRight className={`w-6 h-6 ${platformHoverChevron} group-hover:translate-x-1 transition-transform`} />
                      </div>
                    </div>
                  </a>
                );
              })}
            </div>
          )}
        </>
      )}

      {nightlies.length > 0 && (
        <div className="mb-12">
          <h2 className="text-2xl font-bold text-white mb-4">Nightly Updates</h2>
          <p className="text-slate-400 mb-6">Unstable preview builds for testing new features.</p>
          <div className="grid gap-4">
            {nightlies.map(nightly => {
              const nAssets = getDownloadAssets(nightly);
              if (nAssets.length === 0) return null;
              
              return nAssets.map(nAsset => {
                let platformTag = "Windows";
                let tagColor = "bg-purple-500/20 text-purple-400 border-purple-500/20";
                
                if (nAsset.name.endsWith('.deb')) {
                  platformTag = "Ubuntu / Debian";
                  tagColor = "bg-orange-500/20 text-orange-400 border-orange-500/20";
                } else if (nAsset.name.endsWith('.tar.gz')) {
                  platformTag = "Linux (Portable)";
                  tagColor = "bg-amber-500/20 text-amber-400 border-amber-500/20";
                }
                
                return (
                  <a 
                    key={`${nightly.id}-${nAsset.name}`}
                    href={nAsset.browser_download_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="group flex flex-col sm:flex-row sm:items-center justify-between gap-4 bg-slate-900/50 border border-slate-800 rounded-xl p-4 transition-all hover:bg-slate-800/60"
                  >
                    <div className="flex items-center gap-4">
                      <div className="bg-slate-800/80 p-2.5 rounded-lg text-slate-400 group-hover:text-white transition-colors">
                        <Download className="w-5 h-5" />
                      </div>
                      <div>
                        <h3 className="text-white font-medium flex items-center gap-2">
                          {nightly.tag_name}
                          <span className={`text-xs px-2 py-0.5 rounded-full border ${tagColor}`}>{platformTag}</span>
                          <span className="text-xs bg-slate-500/20 text-slate-400 px-2 py-0.5 rounded-full border border-slate-500/20">Nightly</span>
                        </h3>
                        <p className="text-slate-500 text-sm mt-0.5">
                          {formatDate(nightly.published_at)} • {formatSize(nAsset.size)} • {nAsset.name}
                        </p>
                      </div>
                    </div>
                    <div className="text-slate-400 group-hover:text-white transition-colors text-sm font-medium">
                      Download
                    </div>
                  </a>
                );
              });
            })}
          </div>
        </div>
      )}

      <h2 className="text-2xl font-bold text-white mb-4 mt-8">Recent Changes</h2>
      <p className="text-slate-400 mb-6">Check out the latest improvements and bug fixes.</p>
      
      <Link to="/docs/changelog" className="inline-flex items-center text-indigo-400 hover:text-indigo-300 font-medium transition group">
        View Full Changelog 
        <span className="ml-1 transition-transform group-hover:translate-x-1">→</span>
      </Link>
    </div>
  );
}
