import { Link } from 'react-router-dom';
import { Scale } from 'lucide-react';
import GitHubIcon from '@mui/icons-material/GitHub';

export function Footer() {
  return (
    <footer className="bg-[#111115] text-slate-400 py-12 border-t border-white/5">
      <div className="max-w-7xl mx-auto px-6">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-8 mb-16">
          <div className="flex flex-col gap-4">
            <h3 className="text-white font-bold tracking-wider text-sm mb-2 uppercase">Legal</h3>
            <Link to="/docs/legal/privacy" className="hover:text-white transition text-sm">Privacy Policy</Link>
            <Link to="/docs/legal/terms" className="hover:text-white transition text-sm">Terms of Service</Link>
            <Link to="/docs/legal/disclaimer" className="hover:text-white transition text-sm">Disclaimer</Link>
            <a href="https://github.com/borneelphukan/nixobdo-pdf/blob/main/LICENSE" target="_blank" rel="noopener noreferrer" className="hover:text-white transition text-sm">License</a>
          </div>
          
          <div className="flex flex-col gap-4">
            <h3 className="text-white font-bold tracking-wider text-sm mb-2 uppercase">Links</h3>
            <a href="https://github.com/borneelphukan/nixobdo-pdf" target="_blank" rel="noopener noreferrer" className="hover:text-white transition text-sm">GitHub</a>
            <Link to="/docs" className="hover:text-white transition text-sm">Documentation</Link>
          </div>
        </div>

        <div className="flex flex-col md:flex-row justify-between items-center gap-6 pt-8 border-t border-white/5">
          <div className="flex flex-col items-center md:items-start gap-1">
            <div className="flex items-center gap-2 text-red-500 mb-1">
              <Scale className="w-8 h-8" />
            </div>
            <span className="text-[10px] tracking-widest uppercase font-bold text-slate-500">MIT LICENSED</span>
          </div>

          <p className="text-sm text-slate-500">© 2026-{new Date().getFullYear()} Borneel Bikash Phukan. All rights reserved.</p>

          <div className="flex items-center gap-4">
            <a href="https://github.com/borneelphukan/nixobdo-pdf" target="_blank" rel="noopener noreferrer" className="text-slate-400 hover:text-white transition">
              <GitHubIcon className="w-5 h-5" />
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
}
