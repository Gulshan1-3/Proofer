import { ProjectFile, FileCommit } from '../types';

const STORAGE_KEY = 'proofer_workspace_v1';

export const DEFAULT_FILES: ProjectFile[] = [
  {
    id: 'file-1',
    name: 'isosceles_base_angles.proof',
    content: `theorem isosceles_base_angles:
    AB = AC -> angle_ABC = angle_ACB
proof
    suppose h1 : AB = AC
    derive h2 : angle_ABC = angle_ACB from h1 using IsoscelesBaseAngles
    therefore angle_ABC = angle_ACB from h2
end`,
    createdAt: Date.now() - 3600000,
    updatedAt: Date.now() - 1800000,
    commits: [
      {
        id: 'c1a7f0e',
        timestamp: Date.now() - 3600000,
        message: 'Initial theorem statement and assumption',
        content: `theorem isosceles_base_angles:\n    AB = AC -> angle_ABC = angle_ACB\nproof\n    suppose h1 : AB = AC\nend`,
        verified: false,
        tag: 'Manual',
      },
      {
        id: 'e89b21a',
        timestamp: Date.now() - 1800000,
        message: 'Kernel certified proof with IsoscelesBaseAngles',
        content: `theorem isosceles_base_angles:\n    AB = AC -> angle_ABC = angle_ACB\nproof\n    suppose h1 : AB = AC\n    derive h2 : angle_ABC = angle_ACB from h1 using IsoscelesBaseAngles\n    therefore angle_ABC = angle_ACB from h2\nend`,
        verified: true,
        tag: 'Verified',
      },
    ],
  },
  {
    id: 'file-2',
    name: 'sss_triangle_congruence.proof',
    content: `theorem sss_congruence:
    AB = DE and BC = EF and CA = FD -> triangle ABC congruent triangle DEF
proof
    suppose h1 : AB = DE and BC = EF and CA = FD
    derive h2 : triangle ABC congruent triangle DEF from h1 using SSS
    therefore triangle ABC congruent triangle DEF from h2
end`,
    createdAt: Date.now() - 7200000,
    updatedAt: Date.now() - 7200000,
    commits: [
      {
        id: 'f3408cd',
        timestamp: Date.now() - 7200000,
        message: 'SSS triangle congruence theorem verified',
        content: `theorem sss_congruence:\n    AB = DE and BC = EF and CA = FD -> triangle ABC congruent triangle DEF\nproof\n    suppose h1 : AB = DE and BC = EF and CA = FD\n    derive h2 : triangle ABC congruent triangle DEF from h1 using SSS\n    therefore triangle ABC congruent triangle DEF from h2\nend`,
        verified: true,
        tag: 'Verified',
      },
    ],
  },
  {
    id: 'file-3',
    name: 'thales_circle_right_angle.pf',
    content: `theorem thales_theorem:
    diameter(AB) and on_circle(C) -> angle_ACB = 90
proof
    suppose h1 : diameter(AB) and on_circle(C)
    derive h2 : angle_ACB = 90 from h1 using AngleSum180
    therefore angle_ACB = 90 from h2
end`,
    createdAt: Date.now() - 10800000,
    updatedAt: Date.now() - 10800000,
    commits: [
      {
        id: 'b5e1974',
        timestamp: Date.now() - 10800000,
        message: 'Inscribed right angle theorem',
        content: `theorem thales_theorem:\n    diameter(AB) and on_circle(C) -> angle_ACB = 90\nproof\n    suppose h1 : diameter(AB) and on_circle(C)\n    derive h2 : angle_ACB = 90 from h1 using AngleSum180\n    therefore angle_ACB = 90 from h2\nend`,
        verified: true,
        tag: 'Verified',
      },
    ],
  },
];

export function loadWorkspace(): ProjectFile[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length > 0) {
        return parsed;
      }
    }
  } catch (err) {
    console.error('Failed to load workspace from localStorage:', err);
  }
  return DEFAULT_FILES;
}

export function saveWorkspace(files: ProjectFile[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(files));
  } catch (err) {
    console.error('Failed to save workspace to localStorage:', err);
  }
}

export function generateShortHash(): string {
  return Math.random().toString(36).substring(2, 9);
}

export function createCommit(
  message: string,
  content: string,
  verified: boolean,
  tag: 'Verified' | 'Checkpoint' | 'Manual' = 'Manual'
): FileCommit {
  return {
    id: generateShortHash(),
    timestamp: Date.now(),
    message: message.trim() || 'Snapshot checkpoint',
    content,
    verified,
    tag,
  };
}
