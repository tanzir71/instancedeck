import { useState } from 'react';
import type { Meta, StoryObj } from '@storybook/react-vite';
import { expect, userEvent, within } from 'storybook/test';
import { GlobalSwitcher } from './GlobalSwitcher';
import { createPreviewClient, emptyPreviewSnapshot } from './previewClient';
import { fixtureSnapshot } from './fixtures';
import type { DesktopClient } from './model';

const meta = {
  title: 'Desktop/Global switcher',
  parameters: {
    layout: 'fullscreen',
    verifyViewport: { width: 640, height: 420 },
  },
} satisfies Meta;
export default meta;
type Story = StoryObj<typeof meta>;
function Example({
  mode = 'ready',
}: {
  mode?: 'ready' | 'loading' | 'error' | 'empty' | 'busy' | 'action-error';
}) {
  const [closed, setClosed] = useState(false);
  const [client] = useState<DesktopClient>(() => {
    const base = createPreviewClient(
      mode === 'empty' ? emptyPreviewSnapshot : fixtureSnapshot,
      true,
    );
    return {
      ...base,
      window: async () => setClosed(true),
      ...(mode === 'loading' ? { snapshot: () => new Promise(() => {}) } : {}),
      ...(mode === 'error'
        ? {
            snapshot: async () => {
              throw new globalThis.Error('Profile metadata is unavailable.');
            },
          }
        : {}),
      ...(mode === 'busy' ? { action: () => new Promise(() => {}) } : {}),
      ...(mode === 'action-error'
        ? {
            action: async () => {
              throw new globalThis.Error(
                'Windows blocked this fixture launch. No process was closed.',
              );
            },
          }
        : {}),
    };
  });
  return closed ? (
    <main>
      <h1>Simulated palette closed</h1>
    </main>
  ) : (
    <GlobalSwitcher client={client} />
  );
}
export const Ready: Story = { render: () => <Example /> };
export const Empty: Story = { render: () => <Example mode="empty" /> };
export const Loading: Story = { render: () => <Example mode="loading" /> };
export const Error: Story = { render: () => <Example mode="error" /> };
export const NoResults: Story = {
  render: () => <Example />,
  play: async ({ canvasElement }) => {
    const body = within(canvasElement.ownerDocument.body);
    await userEvent.type(body.getByRole('combobox'), 'no fixture match');
    await expect(body.getByText('No matching profiles')).toBeVisible();
  },
};
export const Busy: Story = {
  render: () => <Example mode="busy" />,
  play: async ({ canvasElement }) => {
    const body = within(canvasElement.ownerDocument.body);
    await expect(body.getAllByRole('option')[0]).toBeVisible();
    await userEvent.click(body.getByRole('combobox'));
    await userEvent.keyboard('{Enter}');
    await expect(body.getByRole('combobox')).toHaveAttribute('readonly');
  },
};
export const ActionError: Story = {
  render: () => <Example mode="action-error" />,
  play: async ({ canvasElement }) => {
    const body = within(canvasElement.ownerDocument.body);
    await expect(body.getAllByRole('option')[0]).toBeVisible();
    await userEvent.click(body.getByRole('combobox'));
    await userEvent.keyboard('{Enter}');
    await expect(body.getByRole('alert')).toHaveTextContent(
      'No process was closed.',
    );
  },
};
export const Keyboard: Story = {
  render: () => <Example />,
  play: async ({ canvasElement }) => {
    const body = within(canvasElement.ownerDocument.body);
    await userEvent.type(body.getByRole('combobox'), 'Research');
    await expect(body.getAllByRole('option')).toHaveLength(1);
    await userEvent.keyboard('{ArrowDown}{Enter}');
    await expect(
      body.getByRole('heading', { name: 'Simulated palette closed' }),
    ).toBeVisible();
  },
};
