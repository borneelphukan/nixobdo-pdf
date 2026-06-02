import { useEffect, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

interface GitHubRelease {
  id: number;
  name: string;
  tag_name: string;
  body: string;
  published_at: string;
}

export function DocsChangelog() {
  const [releases, setReleases] = useState<GitHubRelease[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchReleases() {
      try {
        const response = await fetch('https://api.github.com/repos/borneelphukan/nixobdo-pdf/releases');
        if (!response.ok) throw new Error('Failed to fetch releases');
        const data = await response.json();
        setReleases(data.slice(0, 5));
      } catch (error) {
        console.error('Error fetching changelog:', error);
      } finally {
        setLoading(false);
      }
    }
    
    fetchReleases();
  }, []);

  const formatDate = (dateString: string) => {
    const options: Intl.DateTimeFormatOptions = { year: 'numeric', month: 'short', day: 'numeric' };
    return new Date(dateString).toLocaleDateString(undefined, options);
  };

  return (
    <div className="max-w-4xl animate-in fade-in duration-500">
      <h1 className="text-4xl font-extrabold text-white mb-6">Changelog</h1>
      <p className="text-lg text-slate-400 mb-12 leading-relaxed">
        All notable changes to the Nixobdo PDF project based on GitHub releases.
      </p>

      {loading ? (
        <div className="flex items-center gap-3 text-slate-400 bg-slate-900/50 border border-slate-800 p-6 rounded-2xl w-fit">
          <div className="animate-spin rounded-full h-5 w-5 border-2 border-indigo-500 border-t-transparent"></div>
          Loading changelog...
        </div>
      ) : (
        <div className="space-y-16">
          {releases.map((release) => (
            <div key={release.id} id={release.tag_name} className="scroll-mt-24 group">
              <div className="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4 mb-6 border-b border-white/10 pb-4">
                <h2 className="text-3xl font-bold text-white m-0 flex items-center gap-2">
                  <a href={`#${release.tag_name}`} className="hover:text-indigo-400 transition-colors flex items-center gap-2">
                    {release.name || release.tag_name}
                    <span className="opacity-0 group-hover:opacity-100 transition-opacity text-slate-600 text-2xl">#</span>
                  </a>
                </h2>
                <span className="text-indigo-300 font-mono text-sm bg-indigo-500/10 px-3 py-1 rounded-full border border-indigo-500/20 w-fit">
                  {formatDate(release.published_at)}
                </span>
              </div>
              <div className="prose prose-invert prose-slate prose-a:text-indigo-400 hover:prose-a:text-indigo-300 max-w-none prose-headings:text-slate-200 prose-headings:font-semibold prose-li:text-slate-300 prose-p:text-slate-300 prose-strong:text-white prose-code:text-indigo-300 prose-code:bg-indigo-500/10 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded-md prose-pre:bg-slate-900/80 prose-pre:border prose-pre:border-slate-800">
                <ReactMarkdown remarkPlugins={[remarkGfm]}>
                  {release.body}
                </ReactMarkdown>
              </div>
            </div>
          ))}
          {releases.length === 0 && (
            <p className="text-slate-400 bg-slate-900/50 border border-slate-800 p-6 rounded-2xl">No releases found.</p>
          )}
        </div>
      )}
    </div>
  );
}
