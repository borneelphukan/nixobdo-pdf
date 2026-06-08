import { AlertTriangle } from 'lucide-react';

export function DocsDisclaimer() {
  return (
    <div className="max-w-4xl animate-in fade-in duration-500 pb-12">
      <h1 className="text-4xl font-extrabold text-white mb-6">Disclaimer</h1>
      
      <div className="bg-amber-900/20 border border-amber-700/50 rounded-xl p-6 mb-10 flex items-start gap-4">
        <AlertTriangle className="w-8 h-8 text-amber-400 shrink-0 mt-1" />
        <div>
          <h2 className="text-xl font-bold text-amber-400 mb-2">No Warranties</h2>
          <p className="text-slate-300 leading-relaxed">
            The software is provided "as is", without warranty of any kind, express or implied, including but not limited to the warranties of merchantability, fitness for a particular purpose and noninfringement.
          </p>
        </div>
      </div>

      <div className="space-y-8 text-slate-300">
        <section>
          <h3 className="text-2xl font-bold text-white mb-4 border-b border-slate-700/50 pb-2">1. Limitation of Liability</h3>
          <p className="mb-4">
            In no event shall the authors or copyright holders be liable for any claim, damages or other liability, whether in an action of contract, tort or otherwise, arising from, out of or in connection with the software or the use or other dealings in the software.
          </p>
        </section>

        <section>
          <h3 className="text-2xl font-bold text-white mb-4 border-b border-slate-700/50 pb-2">2. Experimental Features</h3>
          <p className="mb-4">
            Some features (such as Nightly builds and format exporters) are experimental and actively in development. You may encounter bugs, crashes, or formatting inconsistencies when utilizing these features.
          </p>
        </section>

        <section>
          <h3 className="text-2xl font-bold text-white mb-4 border-b border-slate-700/50 pb-2">3. External Links</h3>
          <p className="mb-4">
            The Nixobdo PDF website and documentation may contain links to external websites that are not provided or maintained by or in any way affiliated with us. Please note that we do not guarantee the accuracy, relevance, timeliness, or completeness of any information on these external websites.
          </p>
        </section>

        <p className="text-sm text-slate-500 pt-8 mt-12 border-t border-slate-800">
          Last updated: {new Date().toLocaleDateString()}
        </p>
      </div>
    </div>
  );
}
