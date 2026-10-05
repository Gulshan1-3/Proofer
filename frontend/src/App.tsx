import React, { useState, useEffect, useCallback, useRef } from 'react';
import { Header } from './components/Header';
import { ConstructionToolbar } from './components/ConstructionToolbar';
import { EditorPanel } from './components/EditorPanel';
import { GeometryCanvas } from './components/GeometryCanvas';
import { ProofInspector } from './components/ProofInspector';
import { ParametricSidebar } from './components/ParametricSidebar';
import { FileExplorer } from './components/FileExplorer';
import { VersionControlPanel } from './components/VersionControlPanel';
import { CommandPalette, CommandItem } from './components/CommandPalette';
import { StatusBar } from './components/StatusBar';
import { verifyProofCode, synthesizeProofStep } from './services/prooferApi';
import { loadWorkspace, saveWorkspace, createCommit } from './services/storage';
import { parseProofCodeToSteps } from './services/stepMapper';
import { VerificationResponse, VisualPoint, ProjectFile, FileCommit } from './types';
import { FileCode, Play, Eye, RotateCcw, Folder, GitBranch, Binary, Sparkles } from 'lucide-react';
import { Navbar, WebsiteView } from './components/website/Navbar';
import { LandingPage } from './components/website/LandingPage';
import { DocsPortal } from './components/website/DocsPortal';
import { ShowcasePage } from './components/website/ShowcasePage';
import { InstallPage } from './components/website/InstallPage';
import { Footer } from './components/website/Footer';

const TEMPLATES: Record<string, string> = {
  isosceles: `theorem isosceles_base_angles:
    AB = AC -> angle_ABC = angle_ACB
proof
    suppose h1 : AB = AC
    derive h2 : angle_ABC = angle_ACB from h1 using IsoscelesBaseAngles
    therefore angle_ABC = angle_ACB from h2
end`,

  thales: `theorem thales_semicircle_right_angle:
    diameter(AB) and on_circle(C) -> angle_ACB = 90
proof
    suppose h1 : diameter(AB) and on_circle(C)
    derive h2 : angle_ACB = 90 from h1 using InscribedAngle
    therefore angle_ACB = 90 from h2
end`,

  cyclic_quad: `theorem cyclic_quad_opposite_angles:
    cyclic(ABCD) -> angle_DAB + angle_BCD = 180
proof
    suppose h1 : cyclic(ABCD)
    derive h2 : angle_DAB + angle_BCD = 180 from h1 using CyclicQuad
    therefore angle_DAB + angle_BCD = 180 from h2
end`,

  parallelogram: `theorem parallelogram_opposite_sides:
    parallelogram(ABCD) -> AB = CD and BC = DA
proof
    suppose h1 : parallelogram(ABCD)
    derive h2 : AB = CD and BC = DA from h1 using ParallelogramOppSides
    therefore AB = CD and BC = DA from h2
end`,

  sss: `theorem sss_congruence:
    AB = DE and BC = EF and CA = FD -> triangle ABC congruent triangle DEF
proof
    suppose h1 : AB = DE and BC = EF and CA = FD
    derive h2 : triangle ABC congruent triangle DEF from h1 using SSS
    therefore triangle ABC congruent triangle DEF from h2
end`,

  angle_sum: `theorem triangle_angle_sum:
    triangle(ABC) -> angle_sum = 180
proof
    suppose h1 : triangle(ABC)
    derive h2 : angle_sum = 180 from h1 using AngleSum180
    therefore angle_sum = 180 from h2
end`,

  vertical_angles: `theorem vertical_angles:
    intersect(line_AB, line_CD) -> angle_AEC = angle_BED
proof
    suppose h1 : intersect(line_AB, line_CD)
    derive h2 : angle_AEC = angle_BED from h1 using VerticalAngles
    therefore angle_AEC = angle_BED from h2
end`,

  logic_identity: `theorem identity_theorem:
    P -> P
proof
    suppose h1 : P
    therefore P from h1
end`,

  propositional_logic: `theorem modus_ponens_identity:
    P and (P -> Q) -> Q
proof
    suppose h1 : P and (P -> Q)
    derive h2 : Q from h1 using ModusPonens
    therefore Q from h2
end`,

  invalid_rule: `theorem invalid_rule_check:
    AB = AC -> angle_ABC = 90
proof
    suppose h1 : AB = AC
    derive h2 : angle_ABC = 90 from h1 using NonExistentRule
    therefore angle_ABC = 90 from h2
end`,

  mathlib_triangles: `theorem isosceles_base_angles:
    AB = AC -> angle_ABC = angle_ACB
proof
    suppose h1 : AB = AC
    derive h2 : angle_ABC = angle_ACB from h1 using IsoscelesBaseAngles
    therefore angle_ABC = angle_ACB from h2
end

theorem triangle_angle_sum:
    triangle(ABC) -> angle_sum = 180
proof
    suppose h1 : triangle(ABC)
    derive h2 : angle_sum = 180 from h1 using TriangleAngleSum
    therefore angle_sum = 180 from h2
end

theorem triangle_midpoint_theorem:
    midpoint(M, AB) and midpoint(N, AC) -> parallel(line_MN, line_BC)
proof
    suppose h1 : midpoint(M, AB) and midpoint(N, AC)
    derive h2 : parallel(line_MN, line_BC) from h1 using MidpointBisects
    therefore parallel(line_MN, line_BC) from h2
end`,

  mathlib_circles: `theorem thales_theorem:
    diameter(AB) and on_circle(C) -> angle_ACB = 90
proof
    suppose h1 : diameter(AB) and on_circle(C)
    derive h2 : angle_ACB = 90 from h1 using InscribedAngle
    therefore angle_ACB = 90 from h2
end

theorem tangent_perpendicular_radius:
    tangent(line_PT, circle_O) and radius(segment_OT, circle_O) -> perpendicular(line_PT, segment_OT)
proof
    suppose h1 : tangent(line_PT, circle_O) and radius(segment_OT, circle_O)
    derive h2 : perpendicular(line_PT, segment_OT) from h1 using TangentPerpendicularRadius
    therefore perpendicular(line_PT, segment_OT) from h2
end`,

  mathlib_quads: `theorem parallelogram_opposite_sides:
    parallelogram(ABCD) -> AB = CD and BC = DA
proof
    suppose h1 : parallelogram(ABCD)
    derive h2 : AB = CD and BC = DA from h1 using ParallelogramOppSides
    therefore AB = CD and BC = DA from h2
end

theorem cyclic_quad_opposite_angles:
    cyclic(ABCD) -> angle_DAB + angle_BCD = 180
proof
    suppose h1 : cyclic(ABCD)
    derive h2 : angle_DAB + angle_BCD = 180 from h1 using CyclicQuad
    therefore angle_DAB + angle_BCD = 180 from h2
end`,

  mathlib_logic: `theorem modus_ponens_identity:
    P and (P -> Q) -> Q
proof
    suppose h1 : P and (P -> Q)
    derive h2 : Q from h1 using ModusPonens
    therefore Q from h2
end

theorem double_negation:
    not(not(P)) -> P
proof
    suppose h1 : not(not(P))
    derive h2 : P from h1 using DoubleNegation
    therefore P from h2
end`,
};

export const isGeometryCode = (code: string): boolean => {
  const lower = code.toLowerCase();
  const geoTerms = [
    'triangle',
    'circle',
    'diameter',
    'radius',
    'quadrilateral',
    'parallelogram',
    'cyclic',
    'angle',
    'line',
    'segment',
    'midpoint',
    'altitude',
    'bisector',
    'congruent',
    'perpendicular',
    'parallel',
    'intersect',
    'collinear',
    'isosceles',
    'thales',
    'sss',
    'sas',
    'anglesum',
  ];
  const hasKeyword = geoTerms.some((t) => lower.includes(t));
  const hasSideEq = /\b[A-Z]{2}\s*=\s*[A-Z]{2}\b/.test(code);
  const hasAngleProp = /angle_[A-Za-z0-9]+/.test(code);
  return hasKeyword || hasSideEq || hasAngleProp;
};

export const App: React.FC = () => {
  // Workspace File System & VCS state
  const [files, setFiles] = useState<ProjectFile[]>(() => loadWorkspace());
  const [activeFileId, setActiveFileId] = useState<string>(() => {
    const loaded = loadWorkspace();
    return loaded[0]?.id || 'file-1';
  });

  const activeFile = files.find((f) => f.id === activeFileId) || files[0];
  const [code, setCode] = useState<string>(activeFile?.content || TEMPLATES.isosceles);

  // Website routing state (Lean-style multi-page portal)
  const parseHashToView = (): { view: WebsiteView; docId?: string } => {
    const rawHash = typeof window !== 'undefined' ? window.location.hash.replace(/^#\/?/, '') : '';
    if (!rawHash || rawHash === 'home' || rawHash === 'landing') return { view: 'landing' };
    if (rawHash === 'playground' || rawHash === 'workstation') return { view: 'playground' };
    if (rawHash === 'showcase') return { view: 'showcase' };
    if (rawHash === 'install') return { view: 'install' };
    if (rawHash.startsWith('docs')) {
      const parts = rawHash.split('/');
      return { view: 'docs', docId: parts[1] || 'intro-philosophy' };
    }
    return { view: 'landing' };
  };

  const [websiteView, setWebsiteView] = useState<WebsiteView>(() => parseHashToView().view);
  const [activeDocSectionId, setActiveDocSectionId] = useState<string>(() => parseHashToView().docId || 'intro-philosophy');

  useEffect(() => {
    const handleHashChange = () => {
      const parsed = parseHashToView();
      setWebsiteView(parsed.view);
      if (parsed.docId) {
        setActiveDocSectionId(parsed.docId);
      }
    };
    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  const handleNavigate = (view: WebsiteView, docSectionId?: string) => {
    setWebsiteView(view);
    if (docSectionId) {
      setActiveDocSectionId(docSectionId);
      window.location.hash = `#docs/${docSectionId}`;
    } else if (view === 'landing') {
      window.location.hash = '#home';
    } else {
      window.location.hash = `#${view}`;
    }
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  const handleSelectDocSection = (id: string) => {
    setActiveDocSectionId(id);
    window.location.hash = `#docs/${id}`;
  };

  // Drawer & Modal toggles
  const [isFilesOpen, setIsFilesOpen] = useState<boolean>(false);
  const [isVcsOpen, setIsVcsOpen] = useState<boolean>(false);
  const [isPaletteOpen, setIsPaletteOpen] = useState<boolean>(false);

  // Verification & daemon state
  const [verification, setVerification] = useState<VerificationResponse | null>(null);
  const [daemonOnline, setDaemonOnline] = useState<boolean>(true);
  const [, setRevision] = useState<number>(1);
  const [selectedStepId, setSelectedStepId] = useState<number | null>(null);
  const [activeTab, setActiveTab] = useState<'proof' | 'parametric'>('proof');

  // Resizable Panel Widths (percentages)
  const [editorWidthPct, setEditorWidthPct] = useState<number>(38);
  const [canvasWidthPct, setCanvasWidthPct] = useState<number>(34);
  const [resizingPanel, setResizingPanel] = useState<'left' | 'right' | null>(null);

  // Parametric geometry state
  const [apexAngle, setApexAngle] = useState<number>(62);
  const [sideLength, setSideLength] = useState<number>(9.5);
  const baseAngle = (180 - apexAngle) / 2;

  // Geometry Points Coordinates (Supports A, B, C for triangles/circles and D for quadrilaterals)
  const [points, setPoints] = useState<VisualPoint[]>([
    { id: 'pt#4', name: 'A', x: 300, y: 110 },
    { id: 'pt#5', name: 'B', x: 130, y: 390 },
    { id: 'pt#6', name: 'C', x: 470, y: 390 },
    { id: 'pt#7', name: 'D', x: 430, y: 180 },
  ]);

  // Canvas visibility override state: 'auto' (driven by isGeometry), 'visible', or 'hidden'
  const [canvasOverride, setCanvasOverride] = useState<'auto' | 'visible' | 'hidden'>('auto');
  const isGeometry = isGeometryCode(code);
  const showCanvas = canvasOverride === 'visible' || (canvasOverride === 'auto' && isGeometry);

  const handleToggleCanvas = () => {
    setCanvasOverride((prev) => {
      if (prev === 'auto') return isGeometry ? 'hidden' : 'visible';
      return prev === 'visible' ? 'hidden' : 'visible';
    });
  };

  // Keep code in sync with activeFile
  const handleSelectFile = (fileId: string) => {
    const target = files.find((f) => f.id === fileId);
    if (target) {
      setActiveFileId(fileId);
      setCode(target.content);
      setSelectedStepId(null);
    }
  };

  const sanitizeFileName = (name: string): string => {
    const trimmed = name.trim();
    const lastDot = trimmed.lastIndexOf('.');
    if (lastDot === -1) {
      return `${trimmed}.proof`;
    }
    const ext = trimmed.substring(lastDot).toLowerCase();
    if (ext !== '.proof' && ext !== '.pf') {
      return `${trimmed.substring(0, lastDot)}.proof`;
    }
    return trimmed;
  };

  const handleCreateFile = (name: string, templateContent?: string) => {
    const validName = sanitizeFileName(name);
    const newFile: ProjectFile = {
      id: 'file-' + Date.now(),
      name: validName,
      content: templateContent || `theorem new_theorem:\n    P -> P\nproof\n    suppose h : P\n    therefore P from h\nend`,
      createdAt: Date.now(),
      updatedAt: Date.now(),
      commits: [
        createCommit('Initial commit', templateContent || 'theorem new_theorem: P -> P proof suppose h : P therefore P from h end', true, 'Manual')
      ],
    };

    const nextFiles = [...files, newFile];
    setFiles(nextFiles);
    setActiveFileId(newFile.id);
    setCode(newFile.content);
    saveWorkspace(nextFiles);
  };

  const handleDeleteFile = (id: string) => {
    if (files.length <= 1) return;
    const nextFiles = files.filter((f) => f.id !== id);
    setFiles(nextFiles);
    if (activeFileId === id) {
      setActiveFileId(nextFiles[0].id);
      setCode(nextFiles[0].content);
    }
    saveWorkspace(nextFiles);
  };

  const handleRenameFile = (id: string, newName: string) => {
    const validName = sanitizeFileName(newName);
    const nextFiles = files.map((f) => {
      if (f.id === id) {
        return { ...f, name: validName, updatedAt: Date.now() };
      }
      return f;
    });
    setFiles(nextFiles);
    saveWorkspace(nextFiles);
  };

  const handleCommit = (message: string) => {
    const isVerified = verification?.verified ?? false;
    const newCommit = createCommit(message, code, isVerified, 'Manual');

    const nextFiles = files.map((f) => {
      if (f.id === activeFileId) {
        return {
          ...f,
          content: code,
          updatedAt: Date.now(),
          commits: [...(f.commits || []), newCommit],
        };
      }
      return f;
    });

    setFiles(nextFiles);
    saveWorkspace(nextFiles);
  };

  const handleCheckout = (commit: FileCommit) => {
    setCode(commit.content);
    const nextFiles = files.map((f) => {
      if (f.id === activeFileId) {
        return { ...f, content: commit.content, updatedAt: Date.now() };
      }
      return f;
    });
    setFiles(nextFiles);
    saveWorkspace(nextFiles);
  };

  // Recalculate coordinates from parametric parameters
  const updateGeometryFromParameters = useCallback((angleA: number, sideLen: number) => {
    const halfRad = ((angleA / 2) * Math.PI) / 180;
    const pxScale = 30; // 30 pixels per cm
    const halfBasePx = sideLen * Math.sin(halfRad) * pxScale;
    const heightPx = sideLen * Math.cos(halfRad) * pxScale;

    const centerX = 300;
    const baseY = 390;
    const apexY = Math.max(60, baseY - heightPx);

    setPoints([
      { id: 'pt#4', name: 'A', x: centerX, y: apexY },
      { id: 'pt#5', name: 'B', x: centerX - halfBasePx, y: baseY },
      { id: 'pt#6', name: 'C', x: centerX + halfBasePx, y: baseY },
      { id: 'pt#7', name: 'D', x: 430, y: 180 },
    ]);
  }, []);

  const handleApexAngleChange = (newAngle: number) => {
    setApexAngle(newAngle);
    updateGeometryFromParameters(newAngle, sideLength);
  };

  const handleSideLengthChange = (newLen: number) => {
    setSideLength(newLen);
    updateGeometryFromParameters(apexAngle, newLen);
  };

  const handleUpdatePoint = (id: string, x: number, y: number) => {
    setPoints((prev) =>
      prev.map((p) => {
        if (p.id === id) {
          return { ...p, x, y };
        }
        return p;
      })
    );
  };

  const handleResetPoints = () => {
    updateGeometryFromParameters(apexAngle, sideLength);
  };

  const handleSelectExample = (key: string) => {
    if (TEMPLATES[key]) {
      setCode(TEMPLATES[key]);
      setSelectedStepId(null);

      // Adjust points specifically tailored for circle or quad
      if (key === 'thales') {
        setPoints([
          { id: 'pt#4', name: 'A', x: 160, y: 250 },
          { id: 'pt#5', name: 'B', x: 440, y: 250 },
          { id: 'pt#6', name: 'C', x: 252, y: 118 },
          { id: 'pt#7', name: 'D', x: 430, y: 180 },
        ]);
      } else if (key === 'cyclic_quad' || key === 'parallelogram') {
        setPoints([
          { id: 'pt#4', name: 'A', x: 200, y: 160 },
          { id: 'pt#5', name: 'B', x: 170, y: 340 },
          { id: 'pt#6', name: 'C', x: 400, y: 360 },
          { id: 'pt#7', name: 'D', x: 430, y: 180 },
        ]);
      } else {
        updateGeometryFromParameters(apexAngle, sideLength);
      }
    }
  };

  const handleLoadExampleToPlayground = (exampleCode: string, isGeo: boolean = true) => {
    setCode(exampleCode);
    setSelectedStepId(null);
    setCanvasOverride(isGeo ? 'visible' : 'auto');
    setWebsiteView('playground');
    window.location.hash = '#playground';

    if (exampleCode.includes('diameter(AB)')) {
      setPoints([
        { id: 'pt#4', name: 'A', x: 160, y: 250 },
        { id: 'pt#5', name: 'B', x: 440, y: 250 },
        { id: 'pt#6', name: 'C', x: 252, y: 118 },
        { id: 'pt#7', name: 'D', x: 430, y: 180 },
      ]);
    } else if (exampleCode.includes('cyclic(') || exampleCode.includes('parallelogram(')) {
      setPoints([
        { id: 'pt#4', name: 'A', x: 200, y: 160 },
        { id: 'pt#5', name: 'B', x: 170, y: 340 },
        { id: 'pt#6', name: 'C', x: 400, y: 360 },
        { id: 'pt#7', name: 'D', x: 430, y: 180 },
      ]);
    } else if (isGeo) {
      updateGeometryFromParameters(apexAngle, sideLength);
    }
  };

  const handleApplyPatch = (patchText: string) => {
    const lines = code.split('\n');
    const thereforeIdx = lines.findIndex((l) => l.trim().startsWith('therefore '));

    if (thereforeIdx !== -1) {
      lines.splice(thereforeIdx, 0, patchText.trimEnd());
      setCode(lines.join('\n'));
    } else {
      setCode(code + '\n' + patchText);
    }
  };

  const handleRemovePatch = (substrings: string[]) => {
    const lines = code.split('\n');
    const filtered = lines.filter((l) => !substrings.some((sub) => l.includes(sub)));
    setCode(filtered.join('\n'));
  };

  const [isSynthesizing, setIsSynthesizing] = useState(false);

  const handleSynthesizeStep = async () => {
    setIsSynthesizing(true);
    try {
      const res = await synthesizeProofStep(code);
      if (res && res.step && res.step.verified_by_kernel) {
        const lines = code.split('\n');
        const endIdx = lines.findIndex((l) => l.trim() === 'end');
        if (endIdx !== -1) {
          lines.splice(endIdx, 0, `    ${res.step.text}`);
        } else {
          lines.push(`    ${res.step.text}`);
          lines.push('end');
        }
        const updated = lines.join('\n');
        setCode(updated);
      }
    } finally {
      setIsSynthesizing(false);
    }
  };

  // Debounced real-time compilation & auto-persistence
  const verifiedRef = useRef(false);

  useEffect(() => {
    const timer = setTimeout(async () => {
      setRevision((r) => r + 1);
      const res = await verifyProofCode(code);
      setVerification(res);

      // Auto-save code to active file
      setFiles((prevFiles) => {
        const next = prevFiles.map((f) => {
          if (f.id === activeFileId) {
            let nextCommits = f.commits || [];
            // Auto-checkpoint when transitioning to verified
            if (res.verified && !verifiedRef.current) {
              const lastCommit = nextCommits[nextCommits.length - 1];
              if (!lastCommit || lastCommit.content !== code) {
                const autoCommit = createCommit('Auto-checkpoint: Kernel verified milestone', code, true, 'Verified');
                nextCommits = [...nextCommits, autoCommit];
              }
            }
            return { ...f, content: code, updatedAt: Date.now(), commits: nextCommits };
          }
          return f;
        });
        saveWorkspace(next);
        return next;
      });

      verifiedRef.current = res.verified;

      // Ping server daemon
      try {
        const pingUrl = typeof window !== 'undefined' && window.location.port === '5173' ? 'http://127.0.0.1:8086/' : '/';
        const ping = await fetch(pingUrl, { method: 'GET', signal: AbortSignal.timeout(1000) });
        setDaemonOnline(ping.ok);
      } catch {
        setDaemonOnline(false);
      }
    }, 15);

    return () => clearTimeout(timer);
  }, [code, activeFileId]);

  // Global Keyboard Shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+K or Ctrl+K: Open Command Palette
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setIsPaletteOpen((prev) => !prev);
      }
      // Cmd+Enter or Ctrl+Enter: Re-verify
      if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
        e.preventDefault();
        verifyProofCode(code).then((res) => setVerification(res));
      }
      // Escape: Close all overlays
      if (e.key === 'Escape') {
        setIsPaletteOpen(false);
        setIsFilesOpen(false);
        setIsVcsOpen(false);
        setSelectedStepId(null);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [code]);

  // Mouse drag resizing for panels
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!resizingPanel) return;
      const totalWidth = window.innerWidth;

      if (resizingPanel === 'left') {
        const newEditorPct = Math.max(20, Math.min(60, (e.clientX / totalWidth) * 100));
        setEditorWidthPct(newEditorPct);
      } else if (resizingPanel === 'right') {
        if (showCanvas) {
          const editorPx = (editorWidthPct / 100) * totalWidth;
          const newCanvasPct = Math.max(20, Math.min(60, ((e.clientX - editorPx) / totalWidth) * 100));
          setCanvasWidthPct(newCanvasPct);
        }
      }
    };

    const handleMouseUp = () => setResizingPanel(null);

    if (resizingPanel) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
    }
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, [resizingPanel, editorWidthPct, showCanvas]);

  const hasMidpoint = code.includes('construct M as midpoint') || code.includes('midpoint of BC');
  const hasAltitude = code.includes('construct H as altitude') || code.includes('line_AH') || code.includes('altitude of BC');
  const hasBisector = code.includes('construct D as bisector') || code.includes('angle_BAD = angle_CAD') || code.includes('bisector of angle_BAC');
  const activeTheorem = verification?.theorems?.[0];
  const rawSteps = activeTheorem?.steps ?? [];
  const steps = parseProofCodeToSteps(code, rawSteps);
  const theoremProven = activeTheorem?.proven ?? '';
  const theoremName = activeTheorem?.name ?? 'theorem';
  const isVerified = verification?.verified ?? false;
  const validStepCount = steps.filter((s) => s.status === 'Valid').length;

  // Command Palette Items
  const paletteCommands: CommandItem[] = [
    {
      id: 'cmd-thales',
      category: 'Theorem',
      label: "Load Thales' Theorem",
      description: 'Inscribed right angle in semicircle',
      icon: <FileCode size={14} />,
      perform: () => handleSelectExample('thales'),
    },
    {
      id: 'cmd-isosceles',
      category: 'Theorem',
      label: 'Load Isosceles Base Angles',
      description: 'Triangle with two equal sides',
      icon: <FileCode size={14} />,
      perform: () => handleSelectExample('isosceles'),
    },
    {
      id: 'cmd-cyclic',
      category: 'Theorem',
      label: 'Load Cyclic Quadrilateral',
      description: 'Opposite angles sum to 180°',
      icon: <FileCode size={14} />,
      perform: () => handleSelectExample('cyclic_quad'),
    },
    {
      id: 'cmd-parallelogram',
      category: 'Theorem',
      label: 'Load Parallelogram Proof',
      description: 'Opposite sides congruence',
      icon: <FileCode size={14} />,
      perform: () => handleSelectExample('parallelogram'),
    },
    {
      id: 'cmd-logic',
      category: 'Theorem',
      label: 'Load Propositional Logic (P → P)',
      description: 'Pure logic non-spatial theorem',
      icon: <Binary size={14} />,
      perform: () => handleSelectExample('logic_identity'),
    },
    {
      id: 'cmd-verify',
      category: 'Action',
      label: 'Run Kernel Verification',
      shortcut: '⌘↵',
      icon: <Play size={14} />,
      perform: () => verifyProofCode(code).then((res) => setVerification(res)),
    },
    {
      id: 'cmd-synth-step',
      category: 'AI Co-Prover',
      label: 'Infill Next Step (Co-Prover)',
      shortcut: '⌘.',
      icon: <Sparkles size={14} style={{ color: '#38bdf8' }} />,
      perform: handleSynthesizeStep,
    },
    {
      id: 'cmd-reset-pts',
      category: 'Action',
      label: 'Reset Geometric Coordinates',
      icon: <RotateCcw size={14} />,
      perform: handleResetPoints,
    },
    {
      id: 'cmd-toggle-canvas',
      category: 'View',
      label: showCanvas ? 'Hide Geometry Canvas (2-Pane)' : 'Show Geometry Canvas (3-Pane)',
      icon: <Eye size={14} />,
      perform: handleToggleCanvas,
    },
    {
      id: 'cmd-toggle-files',
      category: 'View',
      label: 'Toggle Project File Explorer',
      icon: <Folder size={14} />,
      perform: () => setIsFilesOpen((v) => !v),
    },
    {
      id: 'cmd-toggle-vcs',
      category: 'View',
      label: 'Toggle Version Control Drawer',
      icon: <GitBranch size={14} />,
      perform: () => setIsVcsOpen((v) => !v),
    },
  ];

  if (websiteView !== 'playground') {
    return (
      <div className="website-wrapper">
        <Navbar 
          activeView={websiteView} 
          onNavigate={handleNavigate} 
          daemonOnline={daemonOnline} 
        />
        {websiteView === 'landing' && (
          <LandingPage 
            onNavigate={handleNavigate} 
            onLoadExampleToPlayground={handleLoadExampleToPlayground} 
          />
        )}
        {websiteView === 'docs' && (
          <DocsPortal 
            activeSectionId={activeDocSectionId} 
            onSelectSection={handleSelectDocSection} 
            onNavigate={handleNavigate} 
            onLoadExampleToPlayground={handleLoadExampleToPlayground} 
          />
        )}
        {websiteView === 'showcase' && (
          <ShowcasePage 
            onNavigate={handleNavigate} 
            onLoadExampleToPlayground={handleLoadExampleToPlayground} 
          />
        )}
        {websiteView === 'install' && (
          <InstallPage 
            onNavigate={handleNavigate} 
          />
        )}
        <Footer onNavigate={handleNavigate} />
      </div>
    );
  }

  return (
    <div className="app-container">
      <Header
        verification={verification}
        daemonOnline={daemonOnline}
        onSelectExample={handleSelectExample}
        activeFileName={activeFile.name}
        filesCount={files.length}
        commitCount={activeFile.commits?.length || 0}
        isFilesOpen={isFilesOpen}
        isVcsOpen={isVcsOpen}
        onToggleFiles={() => setIsFilesOpen((v) => !v)}
        onToggleVcs={() => setIsVcsOpen((v) => !v)}
        showCanvas={showCanvas}
        onToggleCanvas={handleToggleCanvas}
        onOpenCommandPalette={() => setIsPaletteOpen(true)}
        onBackToWebsite={() => handleNavigate('landing')}
      />

      <ConstructionToolbar
        onApplyPatch={handleApplyPatch}
        onRemovePatch={handleRemovePatch}
        onResetPoints={handleResetPoints}
        onSynthesizeStep={handleSynthesizeStep}
        isSynthesizing={isSynthesizing}
        hasMidpoint={hasMidpoint}
        hasAltitude={hasAltitude}
        hasBisector={hasBisector}
        isGeometry={isGeometry}
      />

      {/* File Explorer Slide-Out Drawer */}
      {isFilesOpen && (
        <FileExplorer
          files={files}
          activeFileId={activeFileId}
          onSelectFile={handleSelectFile}
          onCreateFile={handleCreateFile}
          onDeleteFile={handleDeleteFile}
          onRenameFile={handleRenameFile}
          onClose={() => setIsFilesOpen(false)}
        />
      )}

      {/* Version Control Slide-Out Drawer */}
      {isVcsOpen && (
        <VersionControlPanel
          file={activeFile}
          onCommit={handleCommit}
          onCheckout={handleCheckout}
          onClose={() => setIsVcsOpen(false)}
        />
      )}

      {/* Keyboard Command Palette */}
      <CommandPalette
        isOpen={isPaletteOpen}
        onClose={() => setIsPaletteOpen(false)}
        commands={paletteCommands}
      />

      {/* Main Workstation 3-Pane / 2-Pane Resizable Layout */}
      <div className={`main-workspace ${showCanvas ? '' : 'canvas-hidden'}`}>
        {/* Left: Code Editor */}
        <div
          style={{
            width: showCanvas ? `${editorWidthPct}%` : '52%',
            display: 'flex',
            flexDirection: 'column',
          }}
        >
          <EditorPanel
            code={code}
            onChange={setCode}
            verification={verification}
            selectedStepId={selectedStepId}
            steps={steps}
            onSelectStep={setSelectedStepId}
          />
        </div>

        {/* Resizer Handle 1: Editor / Canvas */}
        {showCanvas && (
          <div
            className={`workspace-resizer ${resizingPanel === 'left' ? 'dragging' : ''}`}
            onMouseDown={() => setResizingPanel('left')}
            title="Drag to resize Editor / Canvas"
          />
        )}

        {/* Center: Vector Geometry Canvas (Hidden for non-geometry proofs) */}
        {showCanvas && (
          <>
            <div
              style={{
                width: `${canvasWidthPct}%`,
                display: 'flex',
                flexDirection: 'column',
              }}
            >
              <GeometryCanvas
                points={points}
                onUpdatePoint={handleUpdatePoint}
                selectedStepId={selectedStepId}
                code={code}
                steps={steps}
                onSelectStep={setSelectedStepId}
                hasMidpoint={hasMidpoint}
                hasAltitude={hasAltitude}
                hasBisector={hasBisector}
                apexAngleDeg={apexAngle}
                baseAngleDeg={baseAngle}
              />
            </div>

            {/* Resizer Handle 2: Canvas / Inspector */}
            <div
              className={`workspace-resizer ${resizingPanel === 'right' ? 'dragging' : ''}`}
              onMouseDown={() => setResizingPanel('right')}
              title="Drag to resize Canvas / Inspector"
            />
          </>
        )}

        {/* Right: Proof Execution Debugger & Continuous Solver */}
        <div
          className="panel inspector-panel"
          style={{
            flex: 1,
          }}
        >
          <div className="inspector-tabs">
            <button
              className={`tab-btn ${activeTab === 'proof' ? 'active' : ''}`}
              onClick={() => setActiveTab('proof')}
            >
              Proof Execution Trace
            </button>
            <button
              className={`tab-btn ${activeTab === 'parametric' ? 'active' : ''}`}
              onClick={() => setActiveTab('parametric')}
            >
              Continuous Solver
            </button>
          </div>

          <div className="tab-content">
            {activeTab === 'proof' ? (
              <ProofInspector
                steps={steps}
                selectedStepId={selectedStepId}
                onSelectStep={setSelectedStepId}
                theoremProven={theoremProven}
                theoremName={theoremName}
                isVerified={isVerified}
              />
            ) : (
              <ParametricSidebar
                apexAngle={apexAngle}
                onApexAngleChange={handleApexAngleChange}
                sideLength={sideLength}
                onSideLengthChange={handleSideLengthChange}
                baseAngle={baseAngle}
              />
            )}
          </div>
        </div>
      </div>

      {/* Workstation Bottom Status Bar */}
      <StatusBar
        verification={verification}
        daemonOnline={daemonOnline}
        activeFileName={activeFile.name}
        isGeometry={isGeometry}
        stepCount={steps.length}
        validStepCount={validStepCount}
      />
    </div>
  );
};
