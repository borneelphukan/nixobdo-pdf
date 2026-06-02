import { Lightbulb, Info } from 'lucide-react';
import { Link } from 'react-router-dom';

export function DocsGettingStarted() {
  return (
    <div className="max-w-3xl animate-in fade-in duration-500">
      <h1 className="text-4xl font-extrabold text-white mb-6">Getting Started</h1>
      <p className="text-lg text-slate-400 mb-10 leading-relaxed">
        Welcome to <strong>Nixobdo PDF</strong>, a fast, modern, and open-source PDF viewer and editor designed for simplicity and performance. This guide will help you download, launch, and configure the application.
      </p>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">Download & Execution</h2>
      <p className="text-slate-300 mb-4">
        Nixobdo PDF is straightforward to set up on your Windows machine!
      </p>
      <ol className="list-decimal list-inside space-y-3 text-slate-300 mb-8 ml-2">
        <li>Go to the <Link to="/docs/download" className="text-indigo-400 hover:text-indigo-300 font-medium">Download page</Link> to download the latest installer.</li>
        <li>After downloading, you will have the <code>.exe</code> or <code>.msi</code> installer file.</li>
        <li>Double-click the downloaded file to run the installer and follow the on-screen prompts.</li>
        <li>Once installed, launch Nixobdo PDF from your Start menu or desktop shortcut.</li>
      </ol>

      <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl overflow-hidden shadow-lg mb-12">
        <div className="bg-slate-800/80 px-4 py-3 flex items-center gap-2 border-b border-slate-700/50">
          <Lightbulb className="w-5 h-5 text-amber-400" />
          <h2 className="font-semibold text-white m-0">No Administrator Privileges Required</h2>
        </div>
        <div className="p-6 text-slate-300 leading-relaxed">
          <p className="m-0">
            For basic PDF viewing and editing, Nixobdo PDF does not require administrative privileges. You can install and run it safely on any standard user account.
          </p>
        </div>
      </div>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">Community</h2>
      <p className="text-slate-300 mb-4">
        Join the Nixobdo PDF community to share feedback, request features, and connect with other developers.
      </p>
      <ul className="list-disc list-inside space-y-2 text-slate-300 mb-6 ml-2">
        <li><strong>GitHub</strong>: Engage with the project directly on our <a href="https://github.com/borneelphukan/nixobdo-pdf" target="_blank" rel="noopener noreferrer" className="text-indigo-400 hover:text-indigo-300 font-medium">GitHub repository</a>.</li>
      </ul>
      <p className="text-slate-300 mb-10">
        For more details, visit the <Link to="/docs/community" className="text-indigo-400 hover:text-indigo-300 font-medium">Community</Link> page.
      </p>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">Contribute</h2>
      <p className="text-slate-300 mb-4">
        If Nixobdo PDF has been helpful to you, here are a few ways you can support the project and help keep it growing:
      </p>
      <ul className="list-disc list-inside space-y-2 text-slate-300 mb-10 ml-2">
        <li><strong>GitHub</strong>: Star the repository, report bugs, or contribute code.</li>
        <li><strong>Share</strong>: Tell your friends and colleagues about Nixobdo PDF!</li>
      </ul>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">Important Best Practices</h2>
      <p className="text-slate-300 mb-4">
        Keep the following tips in mind to get the best experience:
      </p>
      <ul className="list-disc list-inside space-y-3 text-slate-300 mb-10 ml-2">
        <li><strong>Keep it Updated:</strong> Check the <Link to="/docs/download" className="text-indigo-400 hover:text-indigo-300 font-medium">Download page</Link> frequently or watch the GitHub repository for new releases to benefit from the latest features and bug fixes.</li>
        <li><strong>Backup Documents:</strong> When editing important PDFs, always ensure you have a backup of the original document just in case.</li>
      </ul>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">Disclaimer</h2>
      <p className="text-slate-300 mb-8 leading-relaxed">
        Nixobdo PDF is an independent, open-source project. Please use it at your own risk. Always ensure you are downloading the software from our official GitHub releases page.
      </p>

      <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl overflow-hidden shadow-lg mb-8">
        <div className="bg-slate-800/80 px-4 py-3 flex items-center gap-2 border-b border-slate-700/50">
          <Info className="w-5 h-5 text-blue-400" />
          <h2 className="font-semibold text-white m-0">Learn more</h2>
        </div>
        <div className="p-6">
          <ul className="list-disc list-inside space-y-3 text-indigo-400 m-0">
            <li><a href="https://github.com/borneelphukan/nixobdo-pdf" target="_blank" rel="noopener noreferrer" className="hover:text-indigo-300 font-medium">License & Source Code</a></li>
          </ul>
        </div>
      </div>
    </div>
  );
}
