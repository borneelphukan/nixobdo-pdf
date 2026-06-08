import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Navbar } from './components/Navbar';
import { Footer } from './components/Footer';
import { Home } from './pages/Home';
import { Docs } from './pages/Docs';
import { DocsDownload } from './pages/DocsDownload';
import { DocsCommunity } from './pages/DocsCommunity';
import { DocsGettingStarted } from './pages/DocsGettingStarted';
import { DocsUninstall } from './pages/DocsUninstall';
import { DocsFAQGeneral } from './pages/DocsFAQGeneral';
import { DocsChangelog } from './pages/DocsChangelog';
import { DocsViewing } from './pages/DocsViewing';
import { DocsShortcuts } from './pages/DocsShortcuts';

export default function App() {
  return (
    <BrowserRouter basename="/nixobdo-pdf">
      <div className="min-h-screen flex flex-col bg-slate-950">
        <Navbar />
        <main className="flex-1">
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/docs/*" element={<Docs />}>
              <Route path="download" element={<DocsDownload />} />
              <Route path="changelog" element={<DocsChangelog />} />
              <Route path="community" element={<DocsCommunity />} />
              <Route path="guides/getting-started" element={<DocsGettingStarted />} />
              <Route path="guides/viewing" element={<DocsViewing />} />
              <Route path="guides/shortcuts" element={<DocsShortcuts />} />
              <Route path="guides/uninstall" element={<DocsUninstall />} />
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
