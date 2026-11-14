import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const { color } = JSON.parse(
  await readFile(
    new URL('../../../design/tokens.json', import.meta.url),
    'utf8',
  ),
);
function luminance(hex: string): number {
  const channels = hex
    .slice(1)
    .match(/../g)!
    .map((part: string) => {
      const v = parseInt(part, 16) / 255;
      return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
    });
  return channels[0]! * 0.2126 + channels[1]! * 0.7152 + channels[2]! * 0.0722;
}
function check(foreground: string, background: string, label: string) {
  const a = luminance(foreground),
    b = luminance(background);
  const ratio = (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
  assert(ratio >= 4.5, `${label}: ${ratio.toFixed(2)}:1 < 4.5:1`);
  console.log(`${label}: ${ratio.toFixed(2)}:1`);
}
for (const [name, entry] of Object.entries(color.profile) as [
  string,
  { value: string },
][])
  check(color.ui.paper.value, entry.value, `white on ${name}`);
for (const surface of ['paper', 'canvas'])
  check(
    color.ui['ink-3'].value,
    color.ui[surface].value,
    `caption on ${surface}`,
  );
check(color.ui.paper.value, color.ui.accent.value, 'primary button');
check(color.ui['accent-on-deep'].value, color.ui.deep.value, 'link on deep');
