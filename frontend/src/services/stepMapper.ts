import { ProofStep, StepKind } from '../types';

/**
 * Formats mathematical expressions into clean standard notation.
 * e.g., "diameter(AB) and on_circle(C) -> angle_ACB = 90" => "diameter(AB) ∧ on_circle(C) → ∠ACB = 90°"
 */
export function formatMathFormula(raw: string): string {
  if (!raw) return '';
  return raw
    .replace(/\b->\b/g, '→')
    .replace(/\band\b/g, '∧')
    .replace(/\bor\b/g, '∨')
    .replace(/\bnot\b/g, '¬')
    .replace(/angle_([A-Za-z0-9_]+)/g, '∠$1')
    .replace(/triangle_([A-Za-z0-9_]+)/g, '△$1')
    .replace(/\bdeg\b/g, '°')
    .replace(/\bperp\b/g, '⊥')
    .replace(/\bparallel\b/g, '∥')
    .replace(/h(\d+)/g, (_, d) => 'h' + toSubscript(d));
}

function toSubscript(digits: string): string {
  const map: Record<string, string> = {
    '0': '₀', '1': '₁', '2': '₂', '3': '₃', '4': '₄',
    '5': '₅', '6': '₆', '7': '₇', '8': '₈', '9': '₉',
  };
  return digits.split('').map(c => map[c] || c).join('');
}

/**
 * Extracts geometric entity references from text.
 * e.g., ["AB", "BC", "C", "M", "H", "D", "angle_ACB"]
 */
export function extractEntities(text: string): string[] {
  const entities = new Set<string>();

  // Segments like AB, BC, AC, CD, DA, AH, AM, AD
  const segMatches = text.match(/\b([A-D][A-D]|[A-D][MHD]|[MHD][A-D])\b/g);
  if (segMatches) {
    segMatches.forEach(s => entities.add(s));
  }

  // Points A, B, C, D, M, H
  const ptMatches = text.match(/\b([A-D]|M|H)\b/g);
  if (ptMatches) {
    ptMatches.forEach(p => entities.add(p));
  }

  // Angles like angle_ABC, angle_BAC
  const angMatches = text.match(/\bangle_([A-Z]{3})\b/g);
  if (angMatches) {
    angMatches.forEach(a => entities.add(a));
  }

  // Constructions
  if (text.includes('midpoint') || text.includes(' M ')) entities.add('M');
  if (text.includes('altitude') || text.includes(' H ')) entities.add('H');
  if (text.includes('bisector') || text.includes(' D ')) entities.add('D');
  if (text.includes('circle') || text.includes('diameter') || text.includes('on_circle')) {
    entities.add('circle');
  }

  return Array.from(entities);
}

/**
 * Enriches raw steps or extracts proof steps directly from proof code lines.
 */
export function parseProofCodeToSteps(code: string, rawSteps: ProofStep[] = []): ProofStep[] {
  const lines = code.split('\n');
  const enriched: ProofStep[] = [];
  let stepIdx = 0;

  for (let lineNum = 1; lineNum <= lines.length; lineNum++) {
    const rawLine = lines[lineNum - 1];
    const trimmed = rawLine.trim();

    if (!trimmed || trimmed.startsWith('#')) continue;

    let kind: StepKind | undefined;
    let hypName: string | undefined;
    let ruleName: string | undefined;
    let conclusion: string | undefined;
    const dependencies: string[] = [];

    if (trimmed.startsWith('suppose ')) {
      kind = 'suppose';
      const match = trimmed.match(/^suppose\s+([a-zA-Z0-9_]+)\s*:\s*(.+)$/);
      if (match) {
        hypName = match[1];
        conclusion = match[2];
      } else {
        conclusion = trimmed.replace(/^suppose\s+/, '');
      }
    } else if (trimmed.startsWith('derive ')) {
      kind = 'derive';
      const match = trimmed.match(/^derive\s+([a-zA-Z0-9_]+)\s*:\s*(.+?)(?:\s+from\s+(.+?))?(?:\s+using\s+(.+))?$/);
      if (match) {
        hypName = match[1];
        conclusion = match[2];
        if (match[3]) {
          dependencies.push(...match[3].split(',').map(s => s.trim()));
        }
        if (match[4]) {
          ruleName = match[4].trim();
        }
      } else {
        conclusion = trimmed.replace(/^derive\s+/, '');
      }
    } else if (trimmed.startsWith('construct ')) {
      kind = 'construct';
      conclusion = trimmed.replace(/^construct\s+/, '');
    } else if (trimmed.startsWith('have ')) {
      kind = 'have';
      const match = trimmed.match(/^have\s+([a-zA-Z0-9_]+)\s*:\s*(.+?)(?:\s+from\s+(.+?))?(?:\s+using\s+(.+))?$/);
      if (match) {
        hypName = match[1];
        conclusion = match[2];
        if (match[3]) dependencies.push(...match[3].split(',').map(s => s.trim()));
        if (match[4]) ruleName = match[4].trim();
      } else {
        conclusion = trimmed.replace(/^have\s+/, '');
      }
    } else if (trimmed.startsWith('therefore ')) {
      kind = 'therefore';
      const match = trimmed.match(/^therefore\s+(.+?)(?:\s+from\s+(.+?))?(?:\s+using\s+(.+))?$/);
      if (match) {
        conclusion = match[1];
        if (match[2]) dependencies.push(...match[2].split(',').map(s => s.trim()));
        if (match[3]) ruleName = match[3].trim();
      } else {
        conclusion = trimmed.replace(/^therefore\s+/, '');
      }
    }

    if (kind) {
      stepIdx++;
      const matchingRaw = rawSteps.find(s => s.id === stepIdx);
      const status = matchingRaw ? matchingRaw.status : 'Valid';
      const entities = extractEntities(trimmed);

      enriched.push({
        id: stepIdx,
        text: trimmed,
        status,
        rule: ruleName || matchingRaw?.rule,
        conclusion: conclusion || matchingRaw?.conclusion,
        kind,
        lineNumber: lineNum,
        hypothesis: hypName,
        dependencies,
        entities,
      });
    }
  }

  return enriched.length > 0 ? enriched : rawSteps;
}
