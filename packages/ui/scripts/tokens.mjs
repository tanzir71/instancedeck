import { readFile, mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import StyleDictionary from 'style-dictionary';

const source = JSON.parse(
  await readFile(
    new URL('../../../design/tokens.json', import.meta.url),
    'utf8',
  ),
);
const tokens = {};
for (const [group, entries] of Object.entries(source.color)) {
  for (const [name, token] of Object.entries(entries)) {
    tokens[group === 'ui' ? name : `${group}-${name}`] = {
      value: token.value,
      type: 'color',
    };
  }
}
for (const [name, value] of Object.entries(source.font))
  tokens[`font-${name}`] = { value, type: 'fontFamily' };
for (const size of source.space)
  tokens[`space-${size}`] = { value: `${size}px`, type: 'dimension' };
for (const [name, value] of Object.entries(source.radius))
  tokens[`radius-${name}`] = { value: `${value}px`, type: 'dimension' };
for (const [name, value] of Object.entries(source.type)) {
  for (const [prop, key] of [
    ['size', 'size'],
    ['weight', 'weight'],
    ['line', 'lineHeight'],
    ['tracking', 'tracking'],
  ]) {
    if (value[key] !== undefined)
      tokens[`type-${name}-${prop}`] = {
        value: `${value[key]}${prop === 'size' ? 'px' : ''}`,
        type: 'string',
      };
  }
}

StyleDictionary.registerFormat({
  name: 'ideck/css',
  format: ({ dictionary }) =>
    `/* Generated from design/tokens.json. Do not edit. */\n:root {\n${dictionary.allTokens.map((t) => `  --${t.name}: ${t.value};`).join('\n')}\n}\n`,
});
StyleDictionary.registerFormat({
  name: 'ideck/ts',
  format: () =>
    `// Generated from design/tokens.json. Do not edit.\nexport const tokens = ${JSON.stringify(source, null, 2)} as const;\n`,
});
const out = new URL('../src/tokens/', import.meta.url);
await mkdir(out, { recursive: true });
const sd = new StyleDictionary({
  tokens,
  platforms: {
    css: {
      buildPath: fileURLToPath(out),
      files: [
        { destination: 'generated.css', format: 'ideck/css' },
        { destination: 'generated.ts', format: 'ideck/ts' },
      ],
    },
  },
});
await sd.buildAllPlatforms();
