import React, { useState } from 'react';
import { FileCode, Plus, Trash2, Download, Edit2, Check, X, Folder, AlertCircle, Search } from 'lucide-react';
import { ProjectFile } from '../types';

const ALLOWED_EXTENSIONS = ['.proof', '.pf'];

const validateFileName = (input: string): { valid: boolean; finalName?: string; error?: string } => {
  const trimmed = input.trim();
  if (!trimmed) {
    return { valid: false, error: 'File name cannot be empty.' };
  }
  const lastDot = trimmed.lastIndexOf('.');
  if (lastDot === -1) {
    return { valid: true, finalName: `${trimmed}.proof` };
  }
  const base = trimmed.substring(0, lastDot).trim();
  if (!base) {
    return { valid: false, error: 'File name must have a prefix before extension.' };
  }
  const ext = trimmed.substring(lastDot).toLowerCase();
  if (!ALLOWED_EXTENSIONS.includes(ext)) {
    return {
      valid: false,
      error: `Invalid extension '${ext}'. Only ${ALLOWED_EXTENSIONS.join(' and ')} files are allowed.`,
    };
  }
  return { valid: true, finalName: `${base}${ext}` };
};

interface FileExplorerProps {
  files: ProjectFile[];
  activeFileId: string;
  onSelectFile: (id: string) => void;
  onCreateFile: (name: string, templateContent?: string) => void;
  onDeleteFile: (id: string) => void;
  onRenameFile: (id: string, newName: string) => void;
  onClose: () => void;
}

export const FileExplorer: React.FC<FileExplorerProps> = ({
  files,
  activeFileId,
  onSelectFile,
  onCreateFile,
  onDeleteFile,
  onRenameFile,
  onClose,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [newFileName, setNewFileName] = useState('');
  const [createError, setCreateError] = useState<string | null>(null);

  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState('');
  const [renameError, setRenameError] = useState<string | null>(null);

  const handleStartCreate = () => {
    setIsCreating(true);
    setNewFileName('');
    setCreateError(null);
  };

  const handleConfirmCreate = (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    const res = validateFileName(newFileName);
    if (!res.valid) {
      setCreateError(res.error || 'Invalid filename');
      return;
    }
    onCreateFile(res.finalName!);
    setIsCreating(false);
    setNewFileName('');
    setCreateError(null);
  };

  const handleStartRename = (file: ProjectFile, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingId(file.id);
    setEditName(file.name);
    setRenameError(null);
  };

  const handleConfirmRename = (id: string, e?: React.FormEvent) => {
    if (e) e.preventDefault();
    const res = validateFileName(editName);
    if (!res.valid) {
      setRenameError(res.error || 'Invalid filename');
      return;
    }
    onRenameFile(id, res.finalName!);
    setEditingId(null);
    setRenameError(null);
  };

  const handleDownload = (file: ProjectFile, e: React.MouseEvent) => {
    e.stopPropagation();
    const blob = new Blob([file.content], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = file.name;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  const filteredFiles = files.filter(f => f.name.toLowerCase().includes(searchQuery.toLowerCase()));

  return (
    <div className="file-explorer-drawer">
      <div className="drawer-header">
        <div className="drawer-title-group">
          <Folder size={14} className="drawer-title-icon" />
          <span className="drawer-title">PROJECT EXPLORER</span>
        </div>
        <button className="drawer-close-btn" onClick={onClose} title="Close (Esc)">
          <X size={14} />
        </button>
      </div>

      {/* Search & Actions Bar */}
      <div className="explorer-search-bar">
        <div className="search-input-wrap">
          <Search size={12} className="search-icon" />
          <input
            type="text"
            className="search-input"
            placeholder="Filter files..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>

        <button className="explorer-new-btn" onClick={handleStartCreate} title="New Proof File">
          <Plus size={13} />
          <span>New File</span>
        </button>
      </div>

      {isCreating && (
        <div className="explorer-create-form-wrap">
          <form onSubmit={handleConfirmCreate}>
            <div className="create-input-row">
              <input
                type="text"
                autoFocus
                placeholder="e.g. pythagoras.proof"
                value={newFileName}
                onChange={(e) => {
                  setNewFileName(e.target.value);
                  if (createError) setCreateError(null);
                }}
                className={`drawer-input ${createError ? 'input-error' : ''}`}
              />
              <button type="submit" className="action-btn-confirm" title="Create">
                <Check size={12} />
              </button>
              <button
                type="button"
                className="action-btn-cancel"
                onClick={() => {
                  setIsCreating(false);
                  setCreateError(null);
                }}
                title="Cancel"
              >
                <X size={12} />
              </button>
            </div>
            {createError ? (
              <div className="explorer-error-msg">
                <AlertCircle size={11} />
                <span>{createError}</span>
              </div>
            ) : (
              <div className="explorer-hint-msg">
                Allowed extensions: .proof, .pf (defaults to .proof)
              </div>
            )}
          </form>
        </div>
      )}

      {/* File List */}
      <div className="file-list">
        {filteredFiles.map((file) => {
          const isActive = file.id === activeFileId;
          const isEditing = editingId === file.id;
          const commitCount = file.commits?.length || 0;
          const isVerified = file.commits?.[file.commits.length - 1]?.verified ?? false;

          return (
            <div
              key={file.id}
              className={`file-item ${isActive ? 'active' : ''}`}
              onClick={() => onSelectFile(file.id)}
            >
              <div className="file-item-left">
                <FileCode size={14} className="file-type-icon" />
                <span className={`file-status-indicator ${isVerified ? 'verified' : 'pending'}`} title={isVerified ? 'Verified' : 'Pending'} />

                {isEditing ? (
                  <div className="file-rename-form" onClick={(e) => e.stopPropagation()}>
                    <div className="rename-input-row">
                      <input
                        type="text"
                        autoFocus
                        value={editName}
                        onChange={(e) => {
                          setEditName(e.target.value);
                          if (renameError) setRenameError(null);
                        }}
                        className={`drawer-input rename-input ${renameError ? 'input-error' : ''}`}
                      />
                      <button
                        className="action-btn-confirm"
                        onClick={(e) => handleConfirmRename(file.id, e)}
                      >
                        <Check size={11} />
                      </button>
                      <button
                        className="action-btn-cancel"
                        onClick={() => {
                          setEditingId(null);
                          setRenameError(null);
                        }}
                      >
                        <X size={11} />
                      </button>
                    </div>
                    {renameError && (
                      <div className="explorer-error-msg">
                        <AlertCircle size={10} />
                        <span>{renameError}</span>
                      </div>
                    )}
                  </div>
                ) : (
                  <div className="file-details">
                    <span className="file-name">{file.name}</span>
                    <span className="file-meta">
                      {commitCount} {commitCount === 1 ? 'commit' : 'commits'}
                    </span>
                  </div>
                )}
              </div>

              {!isEditing && (
                <div className="file-actions">
                  <button
                    className="file-action-btn"
                    onClick={(e) => handleDownload(file, e)}
                    title="Export file"
                  >
                    <Download size={12} />
                  </button>
                  <button
                    className="file-action-btn"
                    onClick={(e) => handleStartRename(file, e)}
                    title="Rename"
                  >
                    <Edit2 size={12} />
                  </button>
                  {files.length > 1 && (
                    <button
                      className="file-action-btn delete"
                      onClick={(e) => {
                        e.stopPropagation();
                        onDeleteFile(file.id);
                      }}
                      title="Delete"
                    >
                      <Trash2 size={12} />
                    </button>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};
