export interface SiteRecipe {
  id: string;
  name: string;
  vendor: string;
  recipeVersion: number;
  status: 'verified' | 'community' | 'experimental';
  launch: { args: string[]; env: Record<string, string>; strategies: string[] };
  limitations: string[];
  verifiedOn: { appVersion: string; os: string; date: string; by: string }[];
  seo: { slug: string; title: string; description: string; intro: string };
}
export const recipes = Object.values(
  import.meta.glob<SiteRecipe>('../../../../packages/recipes/*.recipe.json', {
    eager: true,
    import: 'default',
  }),
).sort((a, b) => a.name.localeCompare(b.name));
export const lettersFor = (id: string) =>
  ({
    'claude-desktop': 'Cl',
    codex: 'Cx',
    vscode: 'VS',
    cursor: 'Cu',
    chrome: 'Ch',
    edge: 'Ed',
    brave: 'Br',
    telegram: 'Tg',
  })[id] ?? id.slice(0, 2);
