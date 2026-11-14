import snapshot from '../data/draft-release.json';
import type { SiteRelease } from './releases';
export interface DraftStatus {
  tag: string;
  version: string;
  createdAt: string;
  commit: string;
  runUrl: string;
  notes: string;
  assetNames: string[];
  verified: true;
}
function parse(data: unknown): DraftStatus | null {
  if (data === null) return null;
  if (typeof data !== 'object') throw new Error('Invalid draft snapshot');
  const item = data as Record<string, unknown>;
  if (
    typeof item.tag !== 'string' ||
    !/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(item.tag) ||
    item.version !== item.tag.slice(1) ||
    typeof item.createdAt !== 'string' ||
    !Number.isFinite(Date.parse(item.createdAt)) ||
    typeof item.commit !== 'string' ||
    !/^[a-f0-9]{40}$/.test(item.commit) ||
    typeof item.runUrl !== 'string' ||
    !/^https:\/\/github.com\/tanzir71\/instancedeck\/actions\/runs\/\d+$/.test(
      item.runUrl,
    ) ||
    typeof item.notes !== 'string' ||
    !Array.isArray(item.assetNames) ||
    item.assetNames.some((name) => typeof name !== 'string') ||
    item.verified !== true
  )
    throw new Error('Invalid verified draft metadata');
  return item as unknown as DraftStatus;
}
const draft = parse(snapshot);
export function visibleDraft(releases: SiteRelease[]) {
  return draft && !releases.some((release) => release.tag_name === draft.tag)
    ? draft
    : null;
}
