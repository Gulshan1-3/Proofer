import React, { useState, useEffect, useRef } from 'react';
import { Search, X } from 'lucide-react';

export interface CommandItem {
  id: string;
  category: 'Theorem' | 'Action' | 'View' | 'File' | 'AI Co-Prover';
  label: string;
  description?: string;
  shortcut?: string;
  icon: React.ReactNode;
  perform: () => void;
}

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  commands: CommandItem[];
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({ isOpen, onClose, commands }) => {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  const filtered = commands.filter(cmd =>
    cmd.label.toLowerCase().includes(query.toLowerCase()) ||
    cmd.category.toLowerCase().includes(query.toLowerCase()) ||
    (cmd.description && cmd.description.toLowerCase().includes(query.toLowerCase()))
  );

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex(prev => (prev + 1) % (filtered.length || 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex(prev => (prev - 1 + (filtered.length || 1)) % (filtered.length || 1));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (filtered[selectedIndex]) {
        filtered[selectedIndex].perform();
        onClose();
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div className="palette-overlay" onClick={onClose}>
      <div className="palette-dialog" onClick={e => e.stopPropagation()}>
        <div className="palette-search-bar">
          <Search size={15} className="palette-search-icon" />
          <input
            ref={inputRef}
            type="text"
            className="palette-input"
            placeholder="Type a command, theorem name, or shortcut..."
            value={query}
            onChange={e => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
          />
          <button className="palette-close-btn" onClick={onClose} title="Close (Esc)">
            <X size={14} />
          </button>
        </div>

        <div className="palette-list">
          {filtered.length === 0 ? (
            <div className="palette-empty">No matching commands found</div>
          ) : (
            filtered.map((cmd, idx) => (
              <div
                key={cmd.id}
                className={`palette-item ${idx === selectedIndex ? 'selected' : ''}`}
                onClick={() => {
                  cmd.perform();
                  onClose();
                }}
                onMouseEnter={() => setSelectedIndex(idx)}
              >
                <div className="palette-item-left">
                  <span className="palette-item-icon">{cmd.icon}</span>
                  <div>
                    <div className="palette-item-label">{cmd.label}</div>
                    {cmd.description && <div className="palette-item-desc">{cmd.description}</div>}
                  </div>
                </div>
                <div className="palette-item-right">
                  <span className="palette-item-category">{cmd.category}</span>
                  {cmd.shortcut && <kbd className="palette-item-shortcut">{cmd.shortcut}</kbd>}
                </div>
              </div>
            ))
          )}
        </div>

        <div className="palette-footer">
          <span><kbd>↑</kbd> <kbd>↓</kbd> Navigate</span>
          <span><kbd>↵</kbd> Execute</span>
          <span><kbd>Esc</kbd> Close</span>
        </div>
      </div>
    </div>
  );
};
