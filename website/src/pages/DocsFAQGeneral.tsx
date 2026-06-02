import { AlertTriangle } from 'lucide-react';
import { Link } from 'react-router-dom';

export function DocsFAQGeneral() {
  return (
    <div className="max-w-3xl animate-in fade-in duration-500">
      <h1 className="text-4xl font-extrabold text-white mb-6">General FAQ</h1>
      <p className="text-lg text-slate-400 mb-10 leading-relaxed">
        Frequently Asked Questions and Answers about Nixobdo PDF in general. Let's cover the basics.
      </p>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">What is Nixobdo PDF?</h2>
      <p className="text-slate-300 mb-8 leading-relaxed">
        Nixobdo PDF is a fast, lightweight, and comprehensive utility designed to help you view, edit, and manage your PDF documents as easily as possible on Windows.
      </p>

      <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl overflow-hidden shadow-lg mb-10">
        <div className="bg-slate-800/80 px-4 py-3 flex items-center gap-2 border-b border-slate-700/50">
          <AlertTriangle className="w-5 h-5 text-red-400" />
          <h2 className="font-semibold text-white m-0">Disclaimer</h2>
        </div>
        <div className="p-6 text-slate-300 leading-relaxed space-y-4">
          <p className="m-0">
            Nixobdo PDF is provided "as is", without warranty of any kind.
          </p>
          <p className="m-0">
            By using this tool, you agree that the authors are not responsible for:
          </p>
          <ul className="list-disc list-inside ml-2 space-y-2">
            <li>Data loss</li>
            <li>Document corruption</li>
            <li>Issues caused by third party software</li>
          </ul>
          <p className="m-0 font-medium text-white pt-2">
            Always keep backups of important documents before modifying them.
          </p>
        </div>
      </div>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">Does this app offer anything other than PDF viewing?</h2>
      <p className="text-slate-300 mb-8 leading-relaxed">
        Yes! The app is built with integrated tools to make your document workflow easier. In addition to lightning-fast viewing, it includes features for annotations and basic editing, so you do not need to download additional software.
      </p>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">Is Nixobdo PDF free?</h2>
      <p className="text-slate-300 mb-8 leading-relaxed">
        Yes, Nixobdo PDF is 100% free and open-source. There are no hidden fees, subscriptions, or paywalls for any of the features.
      </p>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">What versions of Windows are supported?</h2>
      <p className="text-slate-300 mb-8 leading-relaxed">
        Currently, Nixobdo PDF officially supports Windows 10 and Windows 11. Older versions like Windows 7 or 8 are not officially supported as they lack the modern APIs required for some features to function optimally.
      </p>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">How can I report bugs, contribute, or request features?</h2>
      <p className="text-slate-300 mb-8 leading-relaxed">
        We welcome community feedback and contributions! You can submit issues, report bugs, or request new features on our <a href="https://github.com/borneelphukan/nixobdo-pdf/issues" target="_blank" rel="noopener noreferrer" className="text-indigo-400 hover:text-indigo-300 font-medium">GitHub Repository</a>. If you'd like to get your hands dirty, be sure to check out our <Link to="/docs/community" className="text-indigo-400 hover:text-indigo-300 font-medium">Community section</Link>.
      </p>
    </div>
  );
}
