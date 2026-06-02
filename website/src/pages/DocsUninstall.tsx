import { Trash2 } from 'lucide-react';
import { Link } from 'react-router-dom';

export function DocsUninstall() {
  return (
    <div className="max-w-3xl animate-in fade-in duration-500">
      <h1 className="text-4xl font-extrabold text-white mb-6">Uninstalling</h1>
      <p className="text-lg text-slate-400 mb-10 leading-relaxed">
        We're sorry to see you go! If Nixobdo PDF isn't working for you or you no longer need it, here is how you can cleanly uninstall the application from your Windows system.
      </p>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">How to Uninstall</h2>
      <ol className="list-decimal list-inside space-y-4 text-slate-300 mb-8 ml-2">
        <li>Open the <strong>Start Menu</strong> on your Windows PC.</li>
        <li>Search for <strong>"Add or remove programs"</strong> and hit Enter.</li>
        <li>Scroll down the list or use the search bar to find <strong>Nixobdo PDF</strong>.</li>
        <li>Click on the application and select <strong>Uninstall</strong>.</li>
        <li>Follow the on-screen uninstaller prompts to completely remove the application and its associated files from your system.</li>
      </ol>

      <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl overflow-hidden shadow-lg mb-12">
        <div className="bg-slate-800/80 px-4 py-3 flex items-center gap-2 border-b border-slate-700/50">
          <Trash2 className="w-5 h-5 text-red-400" />
          <h2 className="font-semibold text-white m-0">Cleaning Up Leftover Files</h2>
        </div>
        <div className="p-6 text-slate-300 leading-relaxed">
          <p className="m-0">
            The standard uninstaller should handle everything. However, if you want to ensure absolutely no configuration files are left behind, you can manually delete the Nixobdo PDF configuration folder located at: <code>%APPDATA%\nixobdo-pdf</code>
          </p>
        </div>
      </div>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">Having Issues?</h2>
      <p className="text-slate-300 mb-4 leading-relaxed">
        If you are experiencing issues and that is the reason you are uninstalling, we'd love the chance to fix it!
      </p>
      <ul className="list-disc list-inside space-y-3 text-slate-300 mb-10 ml-2">
        <li>Check the <Link to="/docs/faq/troubleshooting" className="text-indigo-400 hover:text-indigo-300 font-medium">Troubleshooting guide</Link> for common fixes.</li>
        <li>Report the bug on our <a href="https://github.com/borneelphukan/nixobdo-pdf/issues" target="_blank" rel="noopener noreferrer" className="text-indigo-400 hover:text-indigo-300 font-medium">GitHub Repository</a> so we can address it in the next update.</li>
      </ul>
    </div>
  );
}
