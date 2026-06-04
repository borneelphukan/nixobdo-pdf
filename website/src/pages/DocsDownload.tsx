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

  const getPrimaryAsset = (release: GitHubRelease): GitHubAsset | null => {
    if (!release.assets || release.assets.length === 0) return null;
    return release.assets.find(a => 
      a.name.endsWith('.exe') || 
      a.name.endsWith('.msi')
    ) || release.assets[0];
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

  if (!latestStable) {
    return (
      <div className="max-w-4xl animate-in fade-in duration-500">
        <h1 className="text-3xl font-bold text-white mb-6">Download</h1>
        <div className="bg-slate-900 border border-slate-800 rounded-2xl p-8 text-slate-400">
          No stable releases found. Check back later!
        </div>
      </div>
    );
  }

  const primaryAsset = getPrimaryAsset(latestStable);
  const totalDownloads = latestStable.assets.reduce((acc, asset) => acc + asset.download_count, 0);

  return (
    <div className="max-w-4xl animate-in fade-in duration-500">
      <div className="mb-8">
        <h1 className="text-3xl font-extrabold text-white mb-4">{latestStable.tag_name}</h1>
        
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

      {primaryAsset && (
        <a 
          href={primaryAsset.browser_download_url}
          target="_blank"
          rel="noopener noreferrer"
          className="group block bg-slate-950/50 border border-purple-400/60 rounded-xl p-4 transition-all hover:bg-slate-900/80 mb-12"
        >
          <div className="flex items-center justify-between gap-4">
            <div className="flex items-center gap-4">
              <div className="bg-slate-800/80 p-3 rounded-lg text-purple-400">
                <Download className="w-6 h-6" />
              </div>
              <div>
                <h3 className="text-white font-semibold text-lg">{primaryAsset.name}</h3>
                <p className="text-slate-500 text-sm mt-0.5">
                  {formatSize(primaryAsset.size)} • {primaryAsset.download_count} downloads
                </p>
              </div>
            </div>
            <div className="pr-2">
              <ChevronRight className="w-6 h-6 text-purple-500 group-hover:translate-x-1 transition-transform" />
            </div>
          </div>
        </a>
      )}

      {nightlies.length > 0 && (
        <div className="mb-12">
          <h2 className="text-2xl font-bold text-white mb-4">Nightly Updates</h2>
          <p className="text-slate-400 mb-6">Unstable preview builds for testing new features.</p>
          <div className="grid gap-4">
            {nightlies.map(nightly => {
              const nAsset = getPrimaryAsset(nightly);
              if (!nAsset) return null;
              return (
                <a 
                  key={nightly.id}
                  href={nAsset.browser_download_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="group flex flex-col sm:flex-row sm:items-center justify-between gap-4 bg-slate-900/50 border border-slate-800 rounded-xl p-4 transition-all hover:bg-slate-800/60"
                >
                  <div className="flex items-center gap-4">
                    <div className="bg-orange-500/10 p-2.5 rounded-lg text-orange-400">
                      <Download className="w-5 h-5" />
                    </div>
                    <div>
                      <h3 className="text-white font-medium flex items-center gap-2">
                        {nightly.tag_name}
                        <span className="text-xs bg-orange-500/20 text-orange-400 px-2 py-0.5 rounded-full border border-orange-500/20">Nightly</span>
                      </h3>
                      <p className="text-slate-500 text-sm mt-0.5">
                        {formatDate(nightly.published_at)} • {formatSize(nAsset.size)}
                      </p>
                    </div>
                  </div>
                  <div className="text-slate-400 group-hover:text-white transition-colors text-sm font-medium">
                    Download
                  </div>
                </a>
              );
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
