import { Keyboard } from 'lucide-react';

export function DocsShortcuts() {
  return (
    <div className="max-w-4xl animate-in fade-in duration-500 pb-12">
      <h1 className="text-4xl font-extrabold text-white mb-6">Keyboard Shortcuts</h1>
      <p className="text-lg text-slate-400 mb-10 leading-relaxed">
        Speed up your workflow and navigate Nixobdo PDF like a pro using these handy keyboard and mouse shortcuts. Here are all the shortcuts currently implemented in the application.
      </p>

      <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl overflow-hidden shadow-lg mb-8">
        <div className="bg-slate-800/80 px-6 py-4 border-b border-slate-700/50 flex items-center gap-3">
          <Keyboard className="w-6 h-6 text-indigo-400" />
          <h2 className="text-xl font-bold text-white m-0">All Shortcuts</h2>
        </div>
        
        <table className="w-full text-left text-sm text-slate-300">
          <thead className="bg-slate-800/40 text-slate-400">
            <tr>
              <th className="px-6 py-4 font-semibold w-1/3">Action</th>
              <th className="px-6 py-4 font-semibold">Shortcut</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-700/50">
            {/* Navigation & Viewing */}
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Zoom In</td>
              <td className="px-6 py-4">
                <div className="flex gap-2">
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Ctrl</kbd>
                  <span className="text-slate-500 font-bold">+</span>
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Mouse Wheel Up</kbd>
                </div>
              </td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Zoom Out</td>
              <td className="px-6 py-4">
                <div className="flex gap-2">
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Ctrl</kbd>
                  <span className="text-slate-500 font-bold">+</span>
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Mouse Wheel Down</kbd>
                </div>
              </td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Smooth Scroll</td>
              <td className="px-6 py-4">
                <div className="flex gap-2">
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Mouse Wheel</kbd>
                </div>
              </td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Page Up</td>
              <td className="px-6 py-4">
                <div className="flex gap-2">
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Arrow Up</kbd>
                  <span className="text-slate-500">or</span>
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Arrow Left</kbd>
                </div>
              </td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Page Down</td>
              <td className="px-6 py-4">
                <div className="flex gap-2">
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Arrow Down</kbd>
                  <span className="text-slate-500">or</span>
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Arrow Right</kbd>
                </div>
              </td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Toggle Fullscreen / Close Search</td>
              <td className="px-6 py-4">
                <div className="flex gap-2">
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Escape</kbd>
                </div>
              </td>
            </tr>

            {/* Document Actions */}
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Save Document</td>
              <td className="px-6 py-4">
                <div className="flex gap-2">
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Ctrl</kbd>
                  <span className="text-slate-500 font-bold">+</span>
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">S</kbd>
                </div>
              </td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Find / Search</td>
              <td className="px-6 py-4">
                <div className="flex gap-2">
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Ctrl</kbd>
                  <span className="text-slate-500 font-bold">+</span>
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">F</kbd>
                </div>
              </td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Select All Text</td>
              <td className="px-6 py-4">
                <div className="flex gap-2">
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Ctrl</kbd>
                  <span className="text-slate-500 font-bold">+</span>
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">A</kbd>
                </div>
              </td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Copy Text</td>
              <td className="px-6 py-4">
                <div className="flex gap-2">
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Ctrl</kbd>
                  <span className="text-slate-500 font-bold">+</span>
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">C</kbd>
                </div>
              </td>
            </tr>

            {/* Annotations */}
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Undo Annotation</td>
              <td className="px-6 py-4">
                <div className="flex gap-2">
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Ctrl</kbd>
                  <span className="text-slate-500 font-bold">+</span>
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Z</kbd>
                </div>
              </td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Redo Annotation</td>
              <td className="px-6 py-4">
                <div className="flex gap-2">
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Ctrl</kbd>
                  <span className="text-slate-500 font-bold">+</span>
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Y</kbd>
                </div>
              </td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Confirm / Enter</td>
              <td className="px-6 py-4">
                <div className="flex gap-2">
                  <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Enter</kbd>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      
      <p className="text-slate-400 mt-8 italic text-sm">
        Note: More shortcuts will be introduced as Nixobdo PDF evolves to add more features. Keep an eye on the <a href="/docs/changelog" className="text-indigo-400 hover:underline">Changelog</a>!
      </p>
    </div>
  );
}
