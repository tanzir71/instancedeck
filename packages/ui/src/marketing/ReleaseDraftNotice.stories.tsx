import type { Meta, StoryObj } from '@storybook/react-vite';
import { ReleaseDraftNotice } from './ReleaseDraftNotice';
const meta = {
  title: 'Marketing/Release draft notice',
  component: ReleaseDraftNotice,
  tags: ['p7'],
  args: {
    tag: 'v0.1.0',
    createdAt: '2026-11-14T00:00:00Z',
    runUrl: 'https://github.com/tanzir71/instancedeck/actions',
    notes:
      'Synthetic release metadata for this Storybook example. No installer is published by the story.',
  },
} satisfies Meta<typeof ReleaseDraftNotice>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Download: Story = {};
export const Changelog: Story = { args: { showNotes: true } };
