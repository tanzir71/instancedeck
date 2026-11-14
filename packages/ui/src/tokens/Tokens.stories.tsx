import type { Meta, StoryObj } from '@storybook/react-vite';
import { tokens } from './generated';
function Tokens() {
  return (
    <main style={{ padding: 32 }}>
      <h1>InstanceDeck foundations</h1>
      <h2 style={{ marginBlock: 24 }}>Profile palette</h2>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 16 }}>
        {Object.entries(tokens.color.profile).map(([name, { value }]) => (
          <div
            key={name}
            style={{
              background: value,
              color: 'var(--paper)',
              width: 144,
              padding: 24,
            }}
          >
            {name}
          </div>
        ))}
      </div>
      <h2 style={{ marginBlock: 24 }}>Type</h2>
      <p style={{ fontFamily: 'var(--font-display)', fontSize: 56 }}>
        Separate accounts. Same desktop.
      </p>
      <p style={{ fontSize: 20 }}>IBM Plex Sans · Body text and interface</p>
      <p style={{ fontFamily: 'var(--font-mono)' }}>
        IBM Plex Mono · Labels and technical details
      </p>
    </main>
  );
}
const meta = { title: 'Foundations/Tokens', component: Tokens } satisfies Meta<
  typeof Tokens
>;
export default meta;
export const Foundations: StoryObj<typeof meta> = {};
