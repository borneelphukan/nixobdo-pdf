import { Link } from 'react-router-dom';
import { FileText } from 'lucide-react';
import GitHubIcon from '@mui/icons-material/GitHub';

export function Navbar() {
  return (
    <nav className="fixed top-0 w-full z-50 border-b border-white/10 bg-slate-950/80 backdrop-blur-xl">
      <div className="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
        <Link to="/" className="flex items-center gap-2 text-white font-bold text-xl hover:opacity-80 transition">
          <FileText className="w-6 h-6 text-indigo-400" />
          Nixobdo PDF
        </Link>
        <div className="flex items-center gap-8 text-sm font-medium text-slate-300">
          <Link to="/docs/guides/getting-started" className="hover:text-white transition">Documentation</Link>
          <a href="https://github.com/borneelphukan/nixobdo-pdf" target="_blank" rel="noopener noreferrer" className="hover:text-white transition">Contribution</a>
          <a href="https://github.com/borneelphukan/nixobdo-pdf" target="_blank" rel="noopener noreferrer" className="hover:text-white transition">
            <GitHubIcon className="w-5 h-5" />
          </a>
        </div>
      </div>
    </nav>
  );
}
