import { Layout, PenTool, FileOutput, FileSignature, MousePointer2 } from 'lucide-react';

export function DocsFeatures() {
  return (
    <div className="max-w-4xl animate-in fade-in duration-500 pb-12">
      <h1 className="text-4xl font-extrabold text-white mb-6">Features</h1>
      <p className="text-lg text-slate-400 mb-10 leading-relaxed">
        Nixobdo PDF comes packed with essential features to make reading, annotating, and exporting your documents a seamless experience. Let's explore what the application can do.
      </p>

      {/* Views */}
      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2 flex items-center gap-2">
        <Layout className="w-6 h-6 text-indigo-400" />
        Views
      </h2>
      <p className="text-slate-300 mb-6">
        Nixobdo PDF offers different viewing modes to suit your reading preferences. Each view is optimized for specific tasks, though some have current limitations.
      </p>
      
      <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-4 mb-8 text-sm text-slate-300">
        <strong className="text-white">How to access:</strong> You can change the viewing mode by navigating to <strong>View &gt; View Options</strong>.
      </div>

      <div className="grid gap-6 md:grid-cols-2 mb-10">
        <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl p-6">
          <h3 className="text-lg font-bold text-white mb-2">Single Page View</h3>
          <p className="text-slate-400 text-sm mb-4">
            Focuses on one page at a time. Ideal for reading documents like books or articles where you want maximum screen real estate dedicated to the current page.
          </p>
          <div className="space-y-2 text-sm">
            <p><span className="text-emerald-400 font-semibold">Benefits:</span> Distraction-free, easier to zoom in on specific details.</p>
            <p><span className="text-rose-400 font-semibold">Limitations:</span> Can feel disconnected when reading documents that frequently refer to previous or next pages.</p>
          </div>
        </div>

        <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl p-6">
          <h3 className="text-lg font-bold text-white mb-2">Continuous Scroll View</h3>
          <p className="text-slate-400 text-sm mb-4">
            Pages are stacked vertically, allowing you to scroll through the entire document smoothly without hard jumps between pages.
          </p>
          <div className="space-y-2 text-sm">
            <p><span className="text-emerald-400 font-semibold">Benefits:</span> Excellent for skimming and maintaining context across page boundaries.</p>
            <p><span className="text-rose-400 font-semibold">Limitations:</span> May consume more memory for very large documents compared to single-page rendering.</p>
          </div>
        </div>
      </div>

      {/* Navigation & Utility */}
      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2 flex items-center gap-2">
        <MousePointer2 className="w-6 h-6 text-indigo-400" />
        Navigation & Utility Bar
      </h2>
      <p className="text-slate-300 mb-6">
        Navigate your documents with ease using the built-in Navigation tools and the quick-access Floating Utility Bar.
      </p>

      <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-4 mb-8 text-sm text-slate-300">
        <strong className="text-white">How to access:</strong> The Utility Bar can be toggled by navigating to <strong>View &gt; Show Utility Bar</strong>.
      </div>

      <div className="grid gap-6 md:grid-cols-2 mb-10">
        <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl p-6">
          <h3 className="text-lg font-bold text-white mb-2">Hand Pan & Pointer Tools</h3>
          <p className="text-slate-400 text-sm mb-4">
            Toggle between the standard Pointer tool for selecting text and the Hand tool for seamlessly clicking and dragging to pan around zoomed-in documents.
          </p>
        </div>

        <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl p-6">
          <h3 className="text-lg font-bold text-white mb-2">Zoom & Scroll Controls</h3>
          <p className="text-slate-400 text-sm mb-4">
            Precisely control your zoom percentage and navigate around the page using the integrated Zoom buttons and directional scroll arrows right from the utility bar.
          </p>
        </div>
      </div>

      {/* Annotations */}
      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2 flex items-center gap-2">
        <PenTool className="w-6 h-6 text-indigo-400" />
        Annotations
      </h2>
      <p className="text-slate-300 mb-6">
        Whether you are studying, reviewing a contract, or collaborating, the annotation tools help you mark up your documents effortlessly.
      </p>

      <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-4 mb-8 text-sm text-slate-300">
        <strong className="text-white">How to access:</strong> Navigate to <strong>Tools &gt; Annotations</strong> on the main toolbar to reveal the specific annotation tools.
      </div>
      
      <div className="bg-slate-900/30 border border-slate-700/30 rounded-xl overflow-hidden mb-10">
        <table className="w-full text-left text-sm text-slate-300">
          <thead className="bg-slate-800/80 text-white">
            <tr>
              <th className="px-6 py-4 font-semibold w-1/4">Tool</th>
              <th className="px-6 py-4 font-semibold">Description</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-700/50">
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Highlight</td>
              <td className="px-6 py-4">Marks selected text with a translucent yellow color (or your chosen color) to emphasize important information.</td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Underline</td>
              <td className="px-6 py-4">Draws a straight line beneath the selected text. Great for emphasizing key terms without covering the text.</td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Strikethrough</td>
              <td className="px-6 py-4">Draws a line through the center of the text, typically used to indicate deletion or correction during review.</td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Freehand Drawing</td>
              <td className="px-6 py-4">Allows you to draw freely on the document using your mouse or stylus. Useful for circling items, drawing arrows, or signing.</td>
            </tr>
            <tr className="hover:bg-slate-800/30 transition-colors">
              <td className="px-6 py-4 font-medium text-white">Text Note</td>
              <td className="px-6 py-4">Adds a small sticky note icon that opens up to reveal typed comments or feedback when clicked.</td>
            </tr>
          </tbody>
        </table>
      </div>

      {/* Signatures */}
      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2 flex items-center gap-2">
        <FileSignature className="w-6 h-6 text-indigo-400" />
        Signatures
      </h2>
      <p className="text-slate-300 mb-6">
        Securely add your digital signatures to any document to authenticate and finalize them without needing to print or use external tools.
      </p>

      <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-4 mb-8 text-sm text-slate-300">
        <strong className="text-white">How to access:</strong> You can add a signature by navigating to <strong>Tools &gt; Signatures</strong>.
      </div>

      {/* Export */}
      <h2 className="text-2xl font-bold text-white mt-12 mb-4 border-b border-white/10 pb-2 flex items-center gap-2">
        <FileOutput className="w-6 h-6 text-indigo-400" />
        Export
      </h2>
      <p className="text-slate-300 mb-6">
        Nixobdo PDF allows you to convert and export your PDF documents into other formats for further editing.
      </p>

      <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-4 mb-8 text-sm text-slate-300">
        <strong className="text-white">How to access:</strong> Navigate to <strong>File &gt; Export As...</strong> and then choose your desired format.
      </div>

      <div className="bg-amber-900/20 border border-amber-700/50 rounded-xl p-6 mb-8">
        <h3 className="text-lg font-bold text-amber-400 mb-3 flex items-center gap-2">
          Current Limitation: DOC & DOCX Export
        </h3>
        <p className="text-slate-300 leading-relaxed mb-4">
          While the application currently supports exporting PDFs to <strong>.doc</strong> and <strong>.docx</strong> formats, this feature is still in early development.
        </p>
        <p className="text-slate-300 leading-relaxed">
          <strong>Note:</strong> The text content will be extracted and saved, but complex layouts, tables, and exact positioning will likely require heavy re-formatting in your word processor.
        </p>
      </div>

    </div>
  );
}
