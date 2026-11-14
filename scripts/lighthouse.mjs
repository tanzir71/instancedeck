import { chromium } from '@playwright/test';
import { createRequire } from 'node:module';
import { spawn } from 'node:child_process';
import { readdir, readFile, writeFile } from 'node:fs/promises';
const require = createRequire(
  new URL('../apps/site/package.json', import.meta.url),
);
const child = spawn(
  process.execPath,
  [
    require.resolve('@lhci/cli/src/cli.js'),
    'autorun',
    '--config=lighthouserc.cjs',
  ],
  {
    cwd: new URL('../apps/site/', import.meta.url),
    env: { ...process.env, CHROME_PATH: chromium.executablePath() },
    stdio: 'inherit',
    windowsHide: true,
  },
);
child.on('error', (error) => {
  console.error(error);
  process.exitCode = 1;
});
child.on('exit', async (code) => {
  process.exitCode = code ?? 1;
  try {
    const directory = new URL('../apps/site/.lighthouseci/', import.meta.url);
    const entries = (await readdir(directory)).filter((name) =>
      /^lhr.*\.json$/.test(name),
    );
    const runs = await Promise.all(
      entries.map(async (name) => {
        const report = JSON.parse(
          await readFile(new URL(name, directory), 'utf8'),
        );
        return {
          url: report.finalDisplayedUrl ?? report.finalUrl,
          scores: Object.fromEntries(
            Object.entries(report.categories).map(([key, value]) => [
              key,
              Math.round(value.score * 100),
            ]),
          ),
          lcpMs: report.audits['largest-contentful-paint'].numericValue,
          cls: report.audits['cumulative-layout-shift'].numericValue,
        };
      }),
    );
    await writeFile(
      new URL('../artifacts/p6/lighthouse-summary.json', import.meta.url),
      JSON.stringify({ passed: code === 0, runs }, null, 2) + '\n',
    );
    console.log(JSON.stringify(runs));
  } catch (error) {
    console.error(error);
    process.exitCode = 1;
  }
});
