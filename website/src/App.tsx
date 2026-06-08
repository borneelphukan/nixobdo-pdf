import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Navbar } from './components/Navbar';
import { Footer } from './components/Footer';
import { Home } from './pages/home';
import { Docs } from './pages/docs';
import { DocsDownload } from './pages/download';
import { DocsCommunity } from './pages/community';
import { DocsGettingStarted } from './pages/getting-started';
import { DocsFAQGeneral } from './pages/faq';
import { DocsChangelog } from './pages/changelog';
import { DocsFeatures } from './pages/features';
import { DocsShortcuts } from './pages/shortcuts';
import { DocsPrivacyPolicy } from './pages/privacy-policy';
import { DocsTermsOfService } from './pages/terms-of-service';
import { DocsDisclaimer } from './pages/disclaimer';

import { ScrollToTop } from './components/ScrollToTop';

export default function App() {
  return (
    <BrowserRouter basename="/nixobdo-pdf">
      <ScrollToTop />
      <div className="min-h-screen flex flex-col bg-slate-950">
        <Navbar />
        <main className="flex-1">
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/docs" element={<Docs />}>
              <Route path="download" element={<DocsDownload />} />
              <Route path="changelog" element={<DocsChangelog />} />
              <Route path="community" element={<DocsCommunity />} />
              <Route path="guides/getting-started" element={<DocsGettingStarted />} />
              <Route path="guides/features" element={<DocsFeatures />} />
              <Route path="guides/shortcuts" element={<DocsShortcuts />} />
              <Route path="legal/privacy" element={<DocsPrivacyPolicy />} />
              <Route path="legal/terms" element={<DocsTermsOfService />} />
              <Route path="legal/disclaimer" element={<DocsDisclaimer />} />
              <Route path="faq/general" element={<DocsFAQGeneral />} />
              <Route path="*" element={
                <div className="max-w-4xl mt-4">
                  <h1 className="text-3xl font-bold text-white mb-6">Documentation Content</h1>
                  <p className="text-slate-400 text-lg">
                    This is a placeholder for the documentation content. Please select a topic from the sidebar.
                  </p>
                </div>
              } />
            </Route>
          </Routes>
        </main>
        <Footer />
      </div>
    </BrowserRouter>
  );
}
