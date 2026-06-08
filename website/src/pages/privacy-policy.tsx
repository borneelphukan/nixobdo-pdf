import { Shield } from 'lucide-react';

export function DocsPrivacyPolicy() {
  return (
    <div className="max-w-4xl animate-in fade-in duration-500 pb-12">
      <h1 className="text-4xl font-extrabold text-white mb-6">Privacy Policy</h1>
      
      <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl p-6 mb-10 flex items-start gap-4">
        <Shield className="w-8 h-8 text-indigo-400 shrink-0 mt-1" />
        <div>
          <h2 className="text-xl font-bold text-white mb-2">Our Commitment to Privacy</h2>
          <p className="text-slate-300 leading-relaxed">
            Nixobdo PDF is designed to run entirely locally on your device. Your privacy is always a priority and it must be ensured that your documents always remain under your control.
          </p>
        </div>
      </div>

      <div className="space-y-8 text-slate-300">
        <section>
          <h3 className="text-2xl font-bold text-white mb-4 border-b border-slate-700/50 pb-2">1. Data Collection</h3>
          <p className="mb-4">
            Nixobdo PDF operates offline. It does <strong>not</strong> collect, store, transmit, or process any of your PDF files, annotations, or personal data on our servers. All processing is done locally on your machine.
          </p>
        </section>

        <section>
          <h3 className="text-2xl font-bold text-white mb-4 border-b border-slate-700/50 pb-2">2. Usage Analytics</h3>
          <p className="mb-4">
           No tracking, telemetry, or analytics tools are used within the Nixobdo PDF application. Your usage habits remain strictly confidential to you.
          </p>
        </section>

        <section>
          <h3 className="text-2xl font-bold text-white mb-4 border-b border-slate-700/50 pb-2">3. Updates</h3>
          <p className="mb-4">
            When you visit our website or download updates, standard server logs (such as IP addresses and browser types) may be temporarily collected by our hosting providers (e.g., GitHub) in accordance with their respective privacy policies.
          </p>
        </section>

        <section>
          <h3 className="text-2xl font-bold text-white mb-4 border-b border-slate-700/50 pb-2">4. Changes to This Policy</h3>
          <p className="mb-4">
            The Privacy Policy may be updated from time to time. Any changes will be posted on this page with an updated revision date.
          </p>
        </section>

        <p className="text-sm text-slate-500 pt-8 mt-12 border-t border-slate-800">
          Last updated: {new Date().toLocaleDateString()}
        </p>
      </div>
    </div>
  );
}
