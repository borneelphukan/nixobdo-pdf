import { Keyboard } from 'lucide-react';

export function DocsShortcuts() {
  return (
    <div className="max-w-3xl animate-in fade-in duration-500">
      <h1 className="text-4xl font-extrabold text-white mb-6">Keyboard Shortcuts</h1>
      <p className="text-lg text-slate-400 mb-10 leading-relaxed">
        Speed up your workflow and navigate Nixobdo PDF like a pro using these handy keyboard and mouse shortcuts.
      </p>

      <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl overflow-hidden shadow-lg">
        <div className="bg-slate-800/80 px-6 py-4 border-b border-slate-700/50 flex items-center gap-3">
          <Keyboard className="w-6 h-6 text-indigo-400" />
          <h2 className="text-xl font-bold text-white m-0">General Shortcuts</h2>
        </div>
        
        <div className="divide-y divide-slate-700/50">
          <div className="flex items-center justify-between p-6 hover:bg-slate-800/30 transition-colors">
            <span className="text-slate-300 font-medium">Zoom In</span>
            <div className="flex gap-2">
              <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Cmd/Ctrl</kbd>
              <span className="text-slate-500 font-bold">+</span>
              <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Mouse Wheel Up</kbd>
            </div>
          </div>

          <div className="flex items-center justify-between p-6 hover:bg-slate-800/30 transition-colors">
            <span className="text-slate-300 font-medium">Zoom Out</span>
            <div className="flex gap-2">
              <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Cmd/Ctrl</kbd>
              <span className="text-slate-500 font-bold">+</span>
              <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Mouse Wheel Down</kbd>
            </div>
          </div>
          
          <div className="flex items-center justify-between p-6 hover:bg-slate-800/30 transition-colors">
            <span className="text-slate-300 font-medium">Smooth Scroll</span>
            <div className="flex gap-2">
              <kbd className="px-2 py-1 bg-slate-800 border border-slate-600 rounded text-sm text-slate-300 font-mono shadow-sm">Mouse Wheel</kbd>
            </div>
          </div>
        </div>
      </div>
      
      <p className="text-slate-400 mt-8 italic text-sm">
        Note: More shortcuts will be introduced as Nixobdo PDF evolves to add more features. Keep an eye on the <a href="/docs/changelog" className="text-indigo-400 hover:underline">Changelog</a>!
      </p>
    </div>
  );
}
