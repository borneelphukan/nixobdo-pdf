import { Link, Outlet } from 'react-router-dom';

export function Docs() {
  return (
    <div className="pt-16 min-h-screen bg-slate-950 text-slate-300 flex selection:bg-indigo-500/30">
      <aside className="w-64 shrink-0 border-r border-white/10 p-6 overflow-y-auto h-[calc(100vh-4rem)] sticky top-16 custom-scrollbar hidden md:block">
        
        <div className="space-y-8">
          <div>
            <h4 className="text-white font-bold mb-4 text-[15px]">General Sections</h4>
            <ul className="space-y-3 text-[14px] text-slate-400">
              <li><Link to="/docs/download" className="hover:text-white transition">Download</Link></li>
              <li><Link to="/docs/changelog" className="hover:text-white transition">Changelog</Link></li>
              <li><Link to="/docs/community" className="hover:text-white transition">Community</Link></li>
            </ul>
          </div>
          
          <div className="w-full h-px bg-white/5"></div>

          <div>
            <h4 className="text-white font-bold mb-4 text-[15px]">FAQ</h4>
            <ul className="space-y-3 text-[14px] text-slate-400">
              <li><Link to="/docs/faq/general" className="hover:text-white transition">General</Link></li>
            </ul>
          </div>

          <div className="w-full h-px bg-white/5"></div>

          <div>
            <h4 className="text-white font-bold mb-4 text-[15px]">Docs</h4>
            <ul className="space-y-3 text-[14px] text-slate-400">
              <li><Link to="/docs/guides/getting-started" className="hover:text-white transition">Getting Started</Link></li>
              <li><Link to="/docs/guides/features" className="hover:text-white transition">Features</Link></li>
              <li><Link to="/docs/guides/shortcuts" className="hover:text-white transition">Shortcuts</Link></li>
            </ul>
          </div>

          <div className="w-full h-px bg-white/5"></div>

          <div>
            <h4 className="text-white font-bold mb-4 text-[15px]">Legal</h4>
            <ul className="space-y-3 text-[14px] text-slate-400">
              <li><Link to="/docs/legal/privacy" className="hover:text-white transition">Privacy Policy</Link></li>
              <li><Link to="/docs/legal/terms" className="hover:text-white transition">Terms of Service</Link></li>
              <li><Link to="/docs/legal/disclaimer" className="hover:text-white transition">Disclaimer</Link></li>
            </ul>
          </div>

          <div className="w-full h-px bg-white/5"></div>
        </div>
      </aside>

      <main className="flex-1 p-8 lg:p-12 overflow-y-auto">
        <Outlet />
      </main>
    </div>
  );
}
