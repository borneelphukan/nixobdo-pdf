import { FileCheck } from 'lucide-react';

export function DocsTermsOfService() {
  return (
    <div className="max-w-4xl animate-in fade-in duration-500 pb-12">
      <h1 className="text-4xl font-extrabold text-white mb-6">Terms of Service</h1>
      
      <div className="bg-slate-900/50 border border-slate-700/50 rounded-xl p-6 mb-10 flex items-start gap-4">
        <FileCheck className="w-8 h-8 text-indigo-400 shrink-0 mt-1" />
        <div>
          <h2 className="text-xl font-bold text-white mb-2">Welcome to Nixobdo PDF</h2>
          <p className="text-slate-300 leading-relaxed">
            By downloading or using Nixobdo PDF, these terms will automatically apply to you. You should make sure that you read them carefully before using the application.
          </p>
        </div>
      </div>

      <div className="space-y-8 text-slate-300">
        <section>
          <h3 className="text-2xl font-bold text-white mb-4 border-b border-slate-700/50 pb-2">1. License and Usage</h3>
          <p className="mb-4">
            Nixobdo PDF is open-source software distributed under the MIT License. You are free to use, modify, and distribute the software in accordance with the license terms. The application is provided free of charge for both personal and commercial use.
          </p>
        </section>

        <section>
          <h3 className="text-2xl font-bold text-white mb-4 border-b border-slate-700/50 pb-2">2. Acceptable Use</h3>
          <p className="mb-4">
            You agree not to use the application in any way that causes, or may cause, damage to the software or impairment of the availability or accessibility of the software, or in any way which is unlawful, illegal, fraudulent, or harmful.
          </p>
        </section>

        <section>
          <h3 className="text-2xl font-bold text-white mb-4 border-b border-slate-700/50 pb-2">3. Intellectual Property Rights</h3>
          <p className="mb-4">
            The intellectual property rights of Nixobdo PDF and its original codebase belong to the project maintainers. However, as it is open-source, you are encouraged to contribute to the project repository.
          </p>
        </section>

        <section>
          <h3 className="text-2xl font-bold text-white mb-4 border-b border-slate-700/50 pb-2">4. Third-Party Libraries</h3>
          <p className="mb-4">
            Nixobdo PDF relies on third-party libraries such as PDFium. You must also abide by the licenses and terms associated with any embedded third-party software components.
          </p>
        </section>

        <p className="text-sm text-slate-500 pt-8 mt-12 border-t border-slate-800">
          Last updated: {new Date().toLocaleDateString()}
        </p>
      </div>
    </div>
  );
}
