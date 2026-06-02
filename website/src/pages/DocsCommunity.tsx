import { Lightbulb, Star, Bug, MessageSquare, Code } from 'lucide-react';

export function DocsCommunity() {
  return (
    <div className="max-w-3xl animate-in fade-in duration-500">
      <h1 className="text-4xl font-extrabold text-white mb-4">Community</h1>
      <p className="text-lg text-slate-400 mb-8">
        Join the Nixobdo PDF community to share feedback, request features, and connect with other users and developers.
      </p>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">GitHub</h2>
      <p className="text-slate-400 mb-6">
        You can engage with the project directly on <a href="https://github.com/borneelphukan/nixobdo-pdf" target="_blank" rel="noopener noreferrer" className="text-indigo-400 hover:text-indigo-300 font-medium">GitHub</a>:
      </p>

      <ul className="space-y-4 mb-12">
        <li className="flex items-start gap-3">
          <Star className="w-5 h-5 text-amber-400 shrink-0 mt-0.5" />
          <span className="text-slate-300 leading-relaxed">
            <strong className="text-white">Star the repository</strong> to show your support and keep track of updates.
          </span>
        </li>
        <li className="flex items-start gap-3">
          <Bug className="w-5 h-5 text-red-400 shrink-0 mt-0.5" />
          <span className="text-slate-300 leading-relaxed">
            <strong className="text-white">Report bugs</strong> by <a href="https://github.com/borneelphukan/nixobdo-pdf/issues/new" target="_blank" rel="noopener noreferrer" className="text-indigo-400 hover:text-indigo-300">creating an issue</a> so we can squash them.
          </span>
        </li>
        <li className="flex items-start gap-3">
          <MessageSquare className="w-5 h-5 text-blue-400 shrink-0 mt-0.5" />
          <span className="text-slate-300 leading-relaxed">
            <strong className="text-white">Request features</strong> by <a href="https://github.com/borneelphukan/nixobdo-pdf/issues/new" target="_blank" rel="noopener noreferrer" className="text-indigo-400 hover:text-indigo-300">opening a feature request</a> if you have ideas for improvement.
          </span>
        </li>
        <li className="flex items-start gap-3">
          <Code className="w-5 h-5 text-emerald-400 shrink-0 mt-0.5" />
          <span className="text-slate-300 leading-relaxed">
            <strong className="text-white">Contribute code</strong> by submitting a pull request. We love community contributions!
          </span>
        </li>
      </ul>

      {/* Open Source Card */}
      <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl overflow-hidden shadow-lg mt-8">
        <div className="bg-slate-800/80 px-4 py-3 flex items-center gap-2 border-b border-slate-700/50">
          <Lightbulb className="w-5 h-5 text-amber-400" />
          <h2 className="font-semibold text-white">Open Source</h2>
        </div>
        <div className="p-6 text-slate-300 leading-relaxed">
          <p>
            Nixobdo PDF is completely free and open source. Contributions of all kinds—code, bug reports, feature requests, and documentation—are warmly welcome! 
          </p>
        </div>
      </div>
    </div>
  );
}
