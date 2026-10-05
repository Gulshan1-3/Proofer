import React from 'react';
import { 
  Play, 
  BookOpen, 
  Compass, 
  Download, 
  Menu,
  X
} from 'lucide-react';

export type WebsiteView = 'landing' | 'docs' | 'playground' | 'showcase' | 'install';

interface NavbarProps {
  activeView: WebsiteView;
  onNavigate: (view: WebsiteView, docSectionId?: string) => void;
  daemonOnline: boolean;
}

export const Navbar: React.FC<NavbarProps> = ({
  activeView,
  onNavigate,
  daemonOnline,
}) => {
  const [mobileMenuOpen, setMobileMenuOpen] = React.useState(false);

  const handleNav = (view: WebsiteView) => {
    onNavigate(view);
    setMobileMenuOpen(false);
  };

  return (
    <header className="site-navbar">
      <div className="site-navbar-container">
        {/* Brand Lockup */}
        <div className="site-brand" onClick={() => handleNav('landing')}>
          <div className="site-logo-symbol">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round">
              <polygon points="12 2 2 22 22 22 12 2" />
              <line x1="12" y1="2" x2="12" y2="22" strokeDasharray="2 2" strokeWidth="1.5" />
            </svg>
          </div>
          <div className="site-brand-text">
            <span className="site-brand-name">PROOFER</span>
            <span className="site-brand-badge">v0.4.2</span>
          </div>
        </div>

        {/* Desktop Navigation Links */}
        <nav className="site-nav-links">
          <button 
            className={`site-nav-btn ${activeView === 'landing' ? 'active' : ''}`}
            onClick={() => handleNav('landing')}
          >
            Overview
          </button>
          <button 
            className={`site-nav-btn ${activeView === 'docs' ? 'active' : ''}`}
            onClick={() => handleNav('docs')}
          >
            <BookOpen size={14} className="nav-icon" />
            Documentation
          </button>
          <button 
            className={`site-nav-btn ${activeView === 'showcase' ? 'active' : ''}`}
            onClick={() => handleNav('showcase')}
          >
            <Compass size={14} className="nav-icon" />
            Showcase
          </button>
          <button 
            className={`site-nav-btn highlight ${activeView === 'playground' ? 'active' : ''}`}
            onClick={() => handleNav('playground')}
          >
            <Play size={13} className="nav-icon text-emerald" />
            Workstation
          </button>
          <button 
            className={`site-nav-btn ${activeView === 'install' ? 'active' : ''}`}
            onClick={() => handleNav('install')}
          >
            <Download size={14} className="nav-icon" />
            Install
          </button>
        </nav>

        {/* Right Side Actions */}
        <div className="site-actions">
          {/* Engine Status */}
          <div className="engine-status-pill" title="Microsecond kernel status">
            <span className="status-dot online"></span>
            <span className="status-label">{daemonOnline ? 'Native Daemon' : 'WASM Kernel'}</span>
          </div>

          {/* GitHub Link */}
          <a 
            href="https://github.com/Gulshan1-3/Proofer" 
            target="_blank" 
            rel="noopener noreferrer"
            className="site-github-btn"
            title="View source on GitHub"
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
            </svg>
            <span className="github-text">GitHub</span>
          </a>

          {/* Primary CTA */}
          <button 
            className="launch-workstation-cta"
            onClick={() => handleNav('playground')}
          >
            <Play size={13} fill="currentColor" />
            <span>Launch Web IDE</span>
          </button>

          {/* Mobile Menu Toggle */}
          <button 
            className="mobile-menu-toggle"
            onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
            aria-label="Toggle Menu"
          >
            {mobileMenuOpen ? <X size={20} /> : <Menu size={20} />}
          </button>
        </div>
      </div>

      {/* Mobile Drawer */}
      {mobileMenuOpen && (
        <div className="mobile-nav-drawer">
          <button className={`mobile-nav-link ${activeView === 'landing' ? 'active' : ''}`} onClick={() => handleNav('landing')}>
            Overview
          </button>
          <button className={`mobile-nav-link ${activeView === 'docs' ? 'active' : ''}`} onClick={() => handleNav('docs')}>
            Documentation
          </button>
          <button className={`mobile-nav-link ${activeView === 'showcase' ? 'active' : ''}`} onClick={() => handleNav('showcase')}>
            Showcase (6 Domains)
          </button>
          <button className={`mobile-nav-link highlight ${activeView === 'playground' ? 'active' : ''}`} onClick={() => handleNav('playground')}>
            Launch Workstation IDE
          </button>
          <button className={`mobile-nav-link ${activeView === 'install' ? 'active' : ''}`} onClick={() => handleNav('install')}>
            Install & Setup
          </button>
          <a href="https://github.com/Gulshan1-3/Proofer" target="_blank" rel="noopener noreferrer" className="mobile-nav-link">
            GitHub Repository
          </a>
        </div>
      )}
    </header>
  );
};
