import { Download, FileText, Code, Settings } from 'lucide-react';
import { Link } from 'react-router-dom';
import GitHubIcon from '@mui/icons-material/GitHub';

export function Home() {
  return (
    <div className="pt-24 min-h-screen bg-slate-950 text-slate-200 font-sans selection:bg-indigo-500/30 relative overflow-hidden flex flex-col justify-between">
      <div className="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] bg-indigo-600/20 rounded-full blur-[120px] pointer-events-none"></div>
      <div className="absolute bottom-[-10%] right-[-10%] w-[50%] h-[50%] bg-blue-600/10 rounded-full blur-[150px] pointer-events-none"></div>
      
      <div className="max-w-7xl mx-auto px-6 pb-20 w-full relative z-10">
        <div className="flex flex-col lg:flex-row gap-12 lg:gap-24 items-center">
          
          {/* Left Hero Content */}
          <div className="flex-1 space-y-8 text-center lg:text-left pt-12">
            <div className="inline-flex items-center justify-center p-4 bg-indigo-500/10 rounded-3xl border border-indigo-500/20 text-indigo-400 mb-2 shadow-2xl shadow-indigo-500/10 backdrop-blur-xl">
              <FileText className="w-12 h-12" />
            </div>
            
            <h1 className="text-5xl lg:text-7xl font-extrabold tracking-tight text-white drop-shadow-sm">
              Nixobdo <span className="text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-blue-400">PDF</span>
            </h1>
            
            <p className="text-lg lg:text-xl text-slate-400 leading-relaxed max-w-xl mx-auto lg:mx-0">
              A fast, modern, and open-source PDF viewer designed for distraction-free reading and ultimate performance.
            </p>
            
            <div className="flex flex-col sm:flex-row items-center justify-center lg:justify-start gap-4 pt-4">
              <Link to="/docs/download" className="flex items-center gap-2 px-6 py-3 bg-indigo-600 hover:bg-indigo-500 rounded-xl font-medium text-white transition-all shadow-lg shadow-indigo-500/25">
                <Download className="w-5 h-5" />
                Download for Windows
              </Link>
              <Link to="/docs/guides/getting-started" className="flex items-center gap-2 px-6 py-3 bg-white/5 hover:bg-white/10 border border-white/10 rounded-xl font-medium text-white transition-all backdrop-blur-sm">
                Get Started
              </Link>
            </div>
          </div>

          {/* Right Hero Image (Logo) */}
          <div className="flex-1 w-full flex justify-center items-center py-10 lg:py-0">
            <div className="relative">
              <div className="absolute inset-0 bg-indigo-500/20 blur-3xl rounded-full scale-150"></div>
              <img src={`${import.meta.env.BASE_URL}logo.svg`} alt="Nixobdo PDF Logo" className="relative z-10 w-64 h-64 lg:w-96 lg:h-96 object-contain drop-shadow-2xl" />
            </div>
          </div>
        </div>

        {/* Info Cards Section */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mt-24">
          
          <div className="bg-slate-900/50 border border-slate-800 rounded-2xl p-8 hover:bg-slate-800/50 transition-colors">
            <div className="bg-white/5 w-12 h-12 rounded-xl flex items-center justify-center mb-6 border border-white/10">
              <GitHubIcon className="w-6 h-6 text-white" />
            </div>
            <h3 className="text-xl font-bold text-white mb-4">Open Source & Community</h3>
            <p className="text-slate-400 mb-8 leading-relaxed">
              Proudly developed on GitHub. Nixobdo PDF is built by and for the community to help everyone enjoy a distraction-free document experience.
            </p>
            <a href="https://github.com/borneelphukan/nixobdo-pdf" target="_blank" rel="noopener noreferrer" className="text-indigo-400 hover:text-indigo-300 font-medium flex items-center gap-1 transition-colors">
              View on GitHub <span aria-hidden="true">&rarr;</span>
            </a>
          </div>

          <div className="bg-slate-900/50 border border-slate-800 rounded-2xl p-8 hover:bg-slate-800/50 transition-colors">
            <div className="bg-white/5 w-12 h-12 rounded-xl flex items-center justify-center mb-6 border border-white/10">
              <Code className="w-6 h-6 text-white" />
            </div>
            <h3 className="text-xl font-bold text-white mb-4">Built with Rust & egui</h3>
            <p className="text-slate-400 mb-8 leading-relaxed">
              Written in Rust and powered by the egui framework. Focuses on blazingly fast performance, extremely low memory footprint, and simplicity.
            </p>
            <a href="https://github.com/borneelphukan/nixobdo-pdf" target="_blank" rel="noopener noreferrer" className="text-indigo-400 hover:text-indigo-300 font-medium flex items-center gap-1 transition-colors">
              View Source <span aria-hidden="true">&rarr;</span>
            </a>
          </div>

          <div className="bg-slate-900/50 border border-slate-800 rounded-2xl p-8 hover:bg-slate-800/50 transition-colors">
            <div className="bg-white/5 w-12 h-12 rounded-xl flex items-center justify-center mb-6 border border-white/10">
              <Settings className="w-6 h-6 text-white" />
            </div>
            <h3 className="text-xl font-bold text-white mb-4">Fast & Distraction-Free</h3>
            <p className="text-slate-400 mb-8 leading-relaxed">
              Optimize your workflow with essential PDF tools, instant loading times, and privacy controls. Reclaims the simple feeling of classic PDF readers.
            </p>
            <Link to="/docs/features/features" className="text-indigo-400 hover:text-indigo-300 font-medium flex items-center gap-1 transition-colors">
              View available features <span aria-hidden="true">&rarr;</span>
            </Link>
          </div>

        </div>
      </div>
    </div>
  );
}
