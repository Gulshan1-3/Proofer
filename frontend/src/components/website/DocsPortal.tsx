import React, { useState, useMemo } from 'react';
import { 
  Search, 
  ChevronRight, 
  Play, 
  Copy, 
  Check, 
  ArrowLeft, 
  ArrowRight,
  Code2
} from 'lucide-react';
import { DOC_CATEGORIES, ALL_DOC_SECTIONS, DocSection, getDocSectionById } from '../../docsData';
import { WebsiteView } from './Navbar';

interface DocsPortalProps {
  activeSectionId: string;
  onSelectSection: (id: string) => void;
  onNavigate: (view: WebsiteView) => void;
  onLoadExampleToPlayground: (code: string, isGeometry: boolean) => void;
}

export const DocsPortal: React.FC<DocsPortalProps> = ({
  activeSectionId,
  onSelectSection,
  onNavigate,
  onLoadExampleToPlayground,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [copiedCodeIndex, setCopiedCodeIndex] = useState<number | null>(null);

  // Active section resolution
  const currentSection: DocSection = useMemo(() => {
    return getDocSectionById(activeSectionId) || ALL_DOC_SECTIONS[0];
  }, [activeSectionId]);

  // Search filtering
  const filteredCategories = useMemo(() => {
    if (!searchQuery.trim()) return DOC_CATEGORIES;
    const q = searchQuery.toLowerCase();

    return DOC_CATEGORIES.map((cat) => {
      const matchingSections = cat.sections.filter(
        (sec) =>
          sec.title.toLowerCase().includes(q) ||
          sec.summary.toLowerCase().includes(q) ||
          sec.keywords.some((k) => k.toLowerCase().includes(q)) ||
          sec.content.toLowerCase().includes(q)
      );
      return {
        ...cat,
        sections: matchingSections,
      };
    }).filter((cat) => cat.sections.length > 0);
  }, [searchQuery]);

  // Previous & Next navigation
  const currentIndex = ALL_DOC_SECTIONS.findIndex((s) => s.id === currentSection.id);
  const prevSection = currentIndex > 0 ? ALL_DOC_SECTIONS[currentIndex - 1] : null;
  const nextSection = currentIndex < ALL_DOC_SECTIONS.length - 1 ? ALL_DOC_SECTIONS[currentIndex + 1] : null;

  const handleCopy = (text: string, index: number) => {
    navigator.clipboard.writeText(text);
    setCopiedCodeIndex(index);
    setTimeout(() => setCopiedCodeIndex(null), 2000);
  };

  const handleRunInWorkstation = (code: string, isGeometry: boolean = true) => {
    onLoadExampleToPlayground(code, isGeometry);
    onNavigate('playground');
  };

  const formatInlineMarkdown = (text: string): React.ReactNode => {
    const parts: React.ReactNode[] = [];
    const regex = /(\*\*[^*]+\*\*|`[^`]+`|\*[^*]+\*)/g;
    let lastIndex = 0;
    let match: RegExpExecArray | null;

    while ((match = regex.exec(text)) !== null) {
      if (match.index > lastIndex) {
        parts.push(text.substring(lastIndex, match.index));
      }
      const token = match[0];
      if (token.startsWith('**') && token.endsWith('**')) {
        parts.push(<strong key={match.index} className="docs-strong">{token.slice(2, -2)}</strong>);
      } else if (token.startsWith('`') && token.endsWith('`')) {
        parts.push(<code key={match.index} className="docs-inline-code">{token.slice(1, -1)}</code>);
      } else if (token.startsWith('*') && token.endsWith('*')) {
        parts.push(<em key={match.index}>{token.slice(1, -1)}</em>);
      }
      lastIndex = regex.lastIndex;
    }

    if (lastIndex < text.length) {
      parts.push(text.substring(lastIndex));
    }

    return parts.length > 0 ? parts : text;
  };

  // Render markdown-like text with code blocks and headers
  const renderFormattedContent = (content: string) => {
    const lines = content.trim().split('\n');
    const elements: React.ReactNode[] = [];
    let inCodeBlock = false;
    let codeLanguage = '';
    let codeBuffer: string[] = [];
    let codeBlockCount = 0;
    let inTable = false;
    let tableRows: string[][] = [];

    const flushTable = (key: number) => {
      if (tableRows.length > 0) {
        const headerRow = tableRows[0];
        const bodyRows = tableRows.slice(2); // Skip separator row
        elements.push(
          <div key={`table-${key}`} className="docs-table-wrapper">
            <table className="docs-table">
              <thead>
                <tr>
                  {headerRow.map((cell, cIdx) => (
                    <th key={cIdx}>{formatInlineMarkdown(cell.trim())}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {bodyRows.map((row, rIdx) => (
                  <tr key={rIdx}>
                    {row.map((cell, cIdx) => (
                      <td key={cIdx}>{formatInlineMarkdown(cell.trim())}</td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        );
        tableRows = [];
        inTable = false;
      }
    };

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];

      // Code blocks
      if (line.trim().startsWith('```')) {
        if (inTable) flushTable(i);
        if (inCodeBlock) {
          // Closing code block
          const fullCode = codeBuffer.join('\n');
          const blockIdx = codeBlockCount++;
          const isProoferCode = codeLanguage.includes('proofer') || fullCode.includes('theorem') || fullCode.includes('proof');

          elements.push(
            <div key={`code-${i}`} className="docs-code-container">
              <div className="docs-code-header">
                <span className="docs-code-lang">{codeLanguage || 'text'}</span>
                <div className="docs-code-actions">
                  <button 
                    className="docs-code-copy-btn"
                    onClick={() => handleCopy(fullCode, blockIdx)}
                    title="Copy code"
                  >
                    {copiedCodeIndex === blockIdx ? <Check size={13} className="text-emerald" /> : <Copy size={13} />}
                    <span>{copiedCodeIndex === blockIdx ? 'Copied' : 'Copy'}</span>
                  </button>
                  {isProoferCode && (
                    <button 
                      className="docs-code-run-btn"
                      onClick={() => handleRunInWorkstation(fullCode, true)}
                      title="Run theorem in live workstation"
                    >
                      <Play size={12} fill="currentColor" />
                      <span>Run in Workstation</span>
                    </button>
                  )}
                </div>
              </div>
              <pre className="docs-code-pre">
                <code>{fullCode}</code>
              </pre>
            </div>
          );
          codeBuffer = [];
          inCodeBlock = false;
        } else {
          // Opening code block
          inCodeBlock = true;
          codeLanguage = line.trim().replace('```', '') || 'text';
        }
        continue;
      }

      if (inCodeBlock) {
        codeBuffer.push(line);
        continue;
      }

      // Tables
      if (line.trim().startsWith('|') && line.trim().endsWith('|')) {
        inTable = true;
        const cells = line.split('|').slice(1, -1);
        tableRows.push(cells);
        continue;
      } else if (inTable) {
        flushTable(i);
      }

      // Headings
      if (line.startsWith('# ')) {
        elements.push(<h1 key={i} className="docs-h1">{formatInlineMarkdown(line.replace('# ', ''))}</h1>);
      } else if (line.startsWith('## ')) {
        elements.push(<h2 key={i} className="docs-h2">{formatInlineMarkdown(line.replace('## ', ''))}</h2>);
      } else if (line.startsWith('### ')) {
        elements.push(<h3 key={i} className="docs-h3">{formatInlineMarkdown(line.replace('### ', ''))}</h3>);
      } else if (line.startsWith('#### ')) {
        elements.push(<h4 key={i} className="docs-h4">{formatInlineMarkdown(line.replace('#### ', ''))}</h4>);
      } else if (line.startsWith('- ') || line.startsWith('* ')) {
        elements.push(
          <li key={i} className="docs-li">
            <span>{formatInlineMarkdown(line.replace(/^[-*]\s+/, ''))}</span>
          </li>
        );
      } else if (line.startsWith('> ')) {
        elements.push(
          <div key={i} className="docs-callout">
            <span className="callout-line">{formatInlineMarkdown(line.replace('> ', ''))}</span>
          </div>
        );
      } else if (line.trim() === '') {
        // Spacer
      } else {
        elements.push(<p key={i} className="docs-p">{formatInlineMarkdown(line)}</p>);
      }
    }

    if (inTable) flushTable(lines.length);

    return elements;
  };

  return (
    <div className="docs-portal-container">
      {/* =========================================================================
          LEFT SIDEBAR: SEARCH & CATEGORY NAVIGATION
          ========================================================================= */}
      <aside className="docs-sidebar">
        <div className="docs-search-box">
          <Search size={15} className="docs-search-icon" />
          <input
            type="text"
            className="docs-search-input"
            placeholder="Search documentation..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
          {searchQuery && (
            <button className="docs-search-clear" onClick={() => setSearchQuery('')}>
              ×
            </button>
          )}
        </div>

        <div className="docs-nav-tree">
          {filteredCategories.map((cat) => (
            <div key={cat.name} className="docs-category-group">
              <div className="docs-category-title">{cat.name}</div>
              <ul className="docs-section-list">
                {cat.sections.map((sec) => (
                  <li key={sec.id}>
                    <button
                      className={`docs-section-btn ${currentSection.id === sec.id ? 'active' : ''}`}
                      onClick={() => onSelectSection(sec.id)}
                    >
                      <span className="section-title-text">{sec.title}</span>
                      {currentSection.id === sec.id && (
                        <ChevronRight size={13} className="section-active-arrow" />
                      )}
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          ))}

          {filteredCategories.length === 0 && (
            <div className="docs-no-results">
              <p>No documentation found matching "{searchQuery}"</p>
              <button className="clear-search-btn" onClick={() => setSearchQuery('')}>
                Clear Search
              </button>
            </div>
          )}
        </div>
      </aside>

      {/* =========================================================================
          CENTER MAIN READING PANE
          ========================================================================= */}
      <main className="docs-main-content">
        {/* Breadcrumb Header */}
        <div className="docs-breadcrumbs">
          <button className="crumb-btn" onClick={() => onNavigate('landing')}>Home</button>
          <span className="crumb-sep">/</span>
          <span className="crumb-cat">{currentSection.category}</span>
          <span className="crumb-sep">/</span>
          <span className="crumb-current">{currentSection.title}</span>
        </div>

        {/* Section Header */}
        <header className="docs-article-header">
          <div className="docs-cat-badge">{currentSection.category}</div>
          <h1 className="docs-article-title">{currentSection.title}</h1>
          <p className="docs-article-lead">{currentSection.summary}</p>
        </header>

        {/* Runnable Example Spotlight Box (if section has one) */}
        {currentSection.runnableExample && (
          <div className="docs-spotlight-example">
            <div className="spotlight-header">
              <div className="spotlight-title">
                <Code2 size={16} className="text-amber" />
                <span>Interactive Example: {currentSection.runnableExample.title}</span>
              </div>
              <button
                className="spotlight-run-btn"
                onClick={() =>
                  handleRunInWorkstation(
                    currentSection.runnableExample!.code,
                    currentSection.runnableExample!.isGeometry
                  )
                }
              >
                <Play size={13} fill="currentColor" />
                <span>Open in Workstation</span>
              </button>
            </div>
            <pre className="spotlight-code">
              <code>{currentSection.runnableExample.code}</code>
            </pre>
          </div>
        )}

        {/* Formatted Article Body */}
        <article className="docs-article-body">
          {renderFormattedContent(currentSection.content)}
        </article>

        {/* Bottom Pagination Links */}
        <nav className="docs-pagination">
          {prevSection ? (
            <button className="pagination-btn prev" onClick={() => onSelectSection(prevSection.id)}>
              <ArrowLeft size={16} />
              <div className="pagination-text">
                <span className="pagination-label">Previous</span>
                <span className="pagination-title">{prevSection.title}</span>
              </div>
            </button>
          ) : (
            <div />
          )}

          {nextSection && (
            <button className="pagination-btn next" onClick={() => onSelectSection(nextSection.id)}>
              <div className="pagination-text">
                <span className="pagination-label">Next</span>
                <span className="pagination-title">{nextSection.title}</span>
              </div>
              <ArrowRight size={16} />
            </button>
          )}
        </nav>
      </main>
    </div>
  );
};
