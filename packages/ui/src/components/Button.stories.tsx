import type { Meta, StoryObj } from '@storybook/react-vite';
import { expect, fn, userEvent, within } from 'storybook/test';
import { Button } from './Button';

const meta = {
  title: 'Components/Button',
  component: Button,
  args: { children: 'Button', onClick: fn() },
  argTypes: {
    kind: {
      control: 'select',
      options: [
        'primary',
        'secondary',
        'inverse',
        'onColor',
        'quiet',
        'danger',
      ],
    },
    size: { control: 'select', options: ['L', 'M'] },
  },
} satisfies Meta<typeof Button>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Primary: Story = {
  play: async ({ canvasElement, args }) => {
    await userEvent.click(within(canvasElement).getByRole('button'));
    await expect(args.onClick).toHaveBeenCalledOnce();
  },
};
export const Secondary: Story = { args: { kind: 'secondary', size: 'M' } };
export const Inverse: Story = {
  args: { kind: 'inverse' },
  globals: { backgrounds: { value: 'dark' } },
};
export const OnColor: Story = {
  args: { kind: 'onColor' },
  decorators: [
    (Story) => (
      <div style={{ background: 'var(--profile-teal)', padding: 24 }}>
        <Story />
      </div>
    ),
  ],
};
export const Quiet: Story = { args: { kind: 'quiet' } };
export const Danger: Story = {
  args: { kind: 'danger', children: 'Delete profile data' },
};
export const Disabled: Story = { args: { disabled: true } };
export const Keyboard: Story = {
  play: async ({ canvasElement, args }) => {
    within(canvasElement).getByRole('button').focus();
    await userEvent.keyboard('{Enter}');
    await expect(args.onClick).toHaveBeenCalledOnce();
  },
};
export const RTL: Story = {
  decorators: [
    (Story) => (
      <div dir="rtl">
        <Story />
      </div>
    ),
  ],
};
