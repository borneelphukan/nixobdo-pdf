import { Download, Monitor, FileText, RefreshCw, Trash2 } from 'lucide-react';
import { Link } from 'react-router-dom';

export function DocsGettingStarted() {
  return (
    <div className="max-w-4xl animate-in fade-in duration-500 pb-12">
      <h1 className="text-4xl font-extrabold text-white mb-6">Getting Started</h1>
      <p className="text-lg text-slate-400 mb-10 leading-relaxed">
        Welcome to Nixobdo PDF! Follow this guide to get the application set up and learn the basics of opening documents and keeping your app up to date.
      </p>

      {/* Download and Install */}
      <h2 id="download" className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2 flex items-center gap-2">
        <Download className="w-6 h-6 text-indigo-400" />
        Download and Install
      </h2>
      <ol className="list-decimal list-inside space-y-3 text-slate-300 mb-8 ml-2">
        <li>Go to the <Link to="/docs/download" className="text-indigo-400 hover:text-indigo-300 font-medium">Download page</Link> to download the latest release for your platform.</li>
        <li>You will receive an <code>.exe</code> installer file. Run it and follow the setup wizard.</li>
        <li>Once installed, you can launch Nixobdo PDF from your Start menu, desktop shortcut, or Applications folder.</li>
      </ol>

      {/* Setting as Default */}
      <h2 id="default" className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2 flex items-center gap-2">
        <Monitor className="w-6 h-6 text-indigo-400" />
        Setting Nixobdo-PDF as Default (Windows Only)
      </h2>
      <p className="text-slate-300 mb-4">
        To make Nixobdo PDF your primary application for opening PDF files automatically on Windows:
      </p>
      <ol className="list-decimal list-inside space-y-3 text-slate-300 mb-8 ml-2">
        <li>Right-click on any <code>.pdf</code> file on your computer.</li>
        <li>Select <strong>Open with</strong> {'>'} <strong>Choose another app</strong>.</li>
        <li>Select <strong>Nixobdo PDF</strong> from the list of applications.</li>
        <li>Check the box that says <strong>Always use this app to open .pdf files</strong>.</li>
        <li>Click <strong>OK</strong>. Your PDF files will now open directly in Nixobdo PDF when double-clicked.</li>
      </ol>

      {/* Opening a Document */}
      <h2 id="opening" className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2 flex items-center gap-2">
        <FileText className="w-6 h-6 text-indigo-400" />
        Opening a Document
      </h2>
      <p className="text-slate-300 mb-4">
        There are several ways to open your documents:
      </p>
      <ul className="list-disc list-inside space-y-3 text-slate-300 mb-8 ml-2">
        <li><strong>Drag and Drop:</strong> The easiest way to open a file is to drag it from your file manager and drop it anywhere into the Nixobdo PDF window.</li>
        <li><strong>File Menu:</strong> Use the menu bar (File {'>'} Open) to launch the native file picker and select your PDF.</li>
        <li><strong>Double-Click:</strong> If you've set Nixobdo PDF as your default viewer, simply double-click any PDF file to open it.</li>
      </ul>

      {/* Checking for Updates */}
      <h2 id="updates" className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2 flex items-center gap-2">
        <RefreshCw className="w-6 h-6 text-indigo-400" />
        Checking for Updates
      </h2>
      <div className="text-slate-300 mb-8 space-y-4">
        <p>
          Updates are frequently released with new features, bug fixes, and performance improvements.
        </p>
        <p>
          You will automatically receive a popup notification if a new update becomes available while using the application.
        </p>
        <p>
          Additionally, you can manually check for new updates at any time by navigating to <strong>Help &gt; Check for Updates</strong> in the top menu bar. You can also view the <Link to="/docs/changelog" className="text-indigo-400 hover:text-indigo-300 font-medium">Changelog</Link> for detailed release notes.
        </p>
      </div>

      {/* Uninstalling */}
      <h2 id="uninstalling" className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2 flex items-center gap-2">
        <Trash2 className="w-6 h-6 text-indigo-400" />
        Uninstalling
      </h2>
      <p className="text-slate-300 mb-4">
        If you need to remove Nixobdo PDF from your system:
      </p>
      <ul className="list-disc list-inside space-y-3 text-slate-300 mb-8 ml-2">
        <li><strong>Windows:</strong> Open <em>Settings {'>'} Apps {'>'} Installed apps</em>, search for Nixobdo PDF, and click <strong>Uninstall</strong>. Alternatively, you can use the Control Panel.</li>
      </ul>
    </div>
  );
}
