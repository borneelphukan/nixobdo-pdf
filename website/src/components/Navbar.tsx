import { Link } from 'react-router-dom';
import { FileText, Sun, Moon } from 'lucide-react';
import GitHubIcon from '@mui/icons-material/GitHub';
import { useState, useEffect } from 'react';

export function Navbar() {
  const [isDark, setIsDark] = useState(() => {
    if (typeof window === 'undefined') {
      return true;
    }
    return (
      localStorage.getItem('theme') === 'dark' ||
        (!('theme' in localStorage) && window.matchMedia('(prefers-color-scheme: dark)').matches)
    );
  });

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  });

  const toggleTheme = () => {
    const newTheme = !isDark;
    setIsDark(newTheme);
    localStorage.setItem('theme', newTheme ? 'dark' : 'light');
  };

  return (
    <nav className="fixed top-0 w-full z-50 border-b border-white/10 bg-slate-950/80 backdrop-blur-xl">
      <div className="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
        <Link to="/" className="flex items-center gap-2 text-white font-bold text-xl hover:opacity-80 transition">
          <FileText className="w-6 h-6 text-indigo-400" />
          Nixobdo PDF
        </Link>
        <div className="flex items-center gap-6 text-sm font-medium text-slate-300">
          <Link to="/docs/guides/getting-started" className="hover:text-white transition hidden md:block">Documentation</Link>
          <a href="https://github.com/borneelphukan/nixobdo-pdf" target="_blank" rel="noopener noreferrer" className="hover:text-white transition hidden md:block">Contribution</a>
          
          <button 
            onClick={toggleTheme} 
            className="p-2 rounded-full hover:bg-slate-800/50 transition-colors text-slate-300 hover:text-white"
            aria-label="Toggle Theme"
          >
            {isDark ? <Sun className="w-5 h-5" /> : <Moon className="w-5 h-5" />}
          </button>

          <a href="https://github.com/borneelphukan/nixobdo-pdf" target="_blank" rel="noopener noreferrer" className="hover:text-white transition">
            <GitHubIcon className="w-5 h-5" />
          </a>
        </div>
      </div>
    </nav>
  );
}
