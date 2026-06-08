import { BookOpen, MousePointer2, ZoomIn } from 'lucide-react';

export function DocsViewing() {
  return (
    <div className="max-w-3xl animate-in fade-in duration-500">
      <h1 className="text-4xl font-extrabold text-white mb-6">Viewing PDFs</h1>
      <p className="text-lg text-slate-400 mb-10 leading-relaxed">
        Nixobdo PDF is designed to provide a distraction-free viewing experience. Learn how to open, navigate, and zoom your documents.
      </p>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">Opening a Document</h2>
      <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl overflow-hidden shadow-lg mb-8">
        <div className="p-6 text-slate-300 leading-relaxed">
          <p className="mb-4">
            You can open a PDF file in Nixobdo PDF using several methods:
          </p>
          <ul className="list-disc list-inside space-y-3 ml-2">
            <li><strong>File Menu:</strong> Click on <span className="font-semibold text-white">File {'>'} Open</span> from the top menu bar to browse and select a PDF file.</li>
            <li><strong>Drag and Drop:</strong> Simply drag a PDF file from your file explorer and drop it into the application window.</li>
            <li><strong>Default Application:</strong> Set Nixobdo PDF as your default PDF viewer to open files by double-clicking them.</li>
          </ul>
        </div>
      </div>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">Navigating Pages</h2>
      <div className="flex gap-4 items-start mb-8 bg-slate-900/30 p-6 rounded-xl border border-slate-700/30">
        <MousePointer2 className="w-8 h-8 text-indigo-400 shrink-0 mt-1" />
        <div className="text-slate-300">
          <h3 className="text-lg font-semibold text-white mb-2">Smooth Scrolling</h3>
          <p className="mb-3">
            Navigation is built to feel natural. Use your <strong>Mouse Wheel</strong> or your laptop's <strong>Trackpad gestures</strong> to smoothly scroll up and down through the pages.
          </p>
          <p>
            Unlike traditional readers with jumpy pagination, Nixobdo PDF offers continuous scrolling so you never lose context between pages.
          </p>
        </div>
      </div>

      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2">Zooming</h2>
      <div className="flex gap-4 items-start mb-8 bg-slate-900/30 p-6 rounded-xl border border-slate-700/30">
        <ZoomIn className="w-8 h-8 text-amber-400 shrink-0 mt-1" />
        <div className="text-slate-300">
          <h3 className="text-lg font-semibold text-white mb-2">Effortless Zoom</h3>
          <p className="mb-3">
            Read comfortably by adjusting the zoom level to fit your screen or preferences:
          </p>
          <ul className="list-disc list-inside space-y-2 ml-2">
            <li>Hold <strong>Cmd/Ctrl</strong> and use your <strong>Mouse Wheel</strong> to zoom in and out dynamically.</li>
            <li>Use the built-in <strong>Zoom slider</strong> in the toolbar for precise control.</li>
            <li>Use the <strong>Zoom In/Out buttons</strong> in the interface for step-by-step adjustments.</li>
          </ul>
        </div>
      </div>

      <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl overflow-hidden shadow-lg mt-12">
        <div className="bg-slate-800/80 px-4 py-3 flex items-center gap-2 border-b border-slate-700/50">
          <BookOpen className="w-5 h-5 text-blue-400" />
          <h2 className="font-semibold text-white m-0">Distraction-Free Focus</h2>
        </div>
        <div className="p-6 text-slate-300 leading-relaxed">
          <p className="m-0">
            Once your document is open and zoomed to your liking, you can focus purely on reading. The UI gets out of your way, giving you the classic, clean PDF experience.
          </p>
        </div>
      </div>
    </div>
  );
}
