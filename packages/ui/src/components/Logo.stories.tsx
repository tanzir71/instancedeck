import type { Meta, StoryObj } from '@storybook/react-vite';
import { Logo } from './Logo';
const meta = { title: 'Components/Logo', component: Logo } satisfies Meta<
  typeof Logo
>;
export default meta;
type Story = StoryObj<typeof meta>;
export const OnLight: Story = {};
export const OnDark: Story = {
  args: { onDark: true },
  decorators: [
    (Story) => (
      <div style={{ background: 'var(--deep)', padding: 24 }}>
        <Story />
      </div>
    ),
  ],
};
