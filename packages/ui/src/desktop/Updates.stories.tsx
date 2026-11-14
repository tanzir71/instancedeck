import { StrictMode, useState } from 'react';
import type { Meta, StoryObj } from '@storybook/react-vite';
import { expect, userEvent, within } from 'storybook/test';
import { UpdateCenter } from './UpdateCenter';
import { DesktopShell } from './DesktopApp';
import { createPreviewClient } from './previewClient';
import { fixtureSnapshot } from './fixtures';
import type { UpdateInfo } from './model';
const meta = {
  title: 'Desktop/Updates',
  parameters: { layout: 'fullscreen' },
  tags: ['p7'],
} satisfies Meta;
export default meta;
type Story = StoryObj<typeof meta>;
function Example({
  mode = 'available',
  dark = false,
}: {
  mode?:
    | 'available'
    | 'current'
    | 'unavailable'
    | 'checking'
    | 'error'
    | 'integrity-error'
    | 'progress'
    | 'portable'
    | 'disabled'
    | 'before-first-run'
    | 'banner';
  dark?: boolean;
}) {
  const [checks, setChecks] = useState(0);
  const [installs, setInstalls] = useState(0);
  const [client] = useState(() => {
    let progress: ((downloaded: number) => void) | undefined;
    return {
      ...createPreviewClient(fixtureSnapshot, true),
      checkUpdates: async (): Promise<UpdateInfo> => {
        setChecks((value) => value + 1);
        if (mode === 'checking') return new Promise(() => {});
        if (mode === 'error')
          throw new Error(
            'Fixture network request failed. No update was downloaded.',
          );
        return {
          status:
            mode === 'current' || mode === 'unavailable' ? mode : 'available',
          version: '0.2.0',
          notes:
            'Fixture release notes. No real installer is used in this story.',
          portable: mode === 'portable',
          currentVersion: '0.1.0',
        };
      },
      updateProgress: async (listener: (downloaded: number) => void) => {
        progress = listener;
        return () => {
          progress = undefined;
        };
      },
      installUpdate: async (_: string, confirmed: boolean) => {
        if (!confirmed)
          throw new Error('Missing explicit fixture confirmation');
        setInstalls((value) => value + 1);
        if (mode === 'integrity-error')
          throw new Error(
            'Fixture integrity signature failed. Nothing was installed.',
          );
        if (mode === 'progress') {
          progress?.(2 * 1024 * 1024);
          return new Promise<void>(() => {});
        }
      },
      openExternal: async () => {},
    };
  });
  const settings = {
    ...fixtureSnapshot.settings,
    allowUpdateChecks: mode !== 'disabled',
    onboardingCompleted: mode !== 'before-first-run',
  };
  return (
    <StrictMode>
      <DesktopShell
        screen="settings"
        theme={dark ? 'dark' : 'light'}
        onNavigate={() => {}}
        onWindow={async () => {}}
      >
        <UpdateCenter
          client={client}
          settings={settings}
          expanded={mode !== 'banner'}
        />
        <span hidden data-testid="fixture-checks">
          {checks}
        </span>
        <span hidden data-testid="fixture-installs">
          {installs}
        </span>
      </DesktopShell>
    </StrictMode>
  );
}
export const Disabled: Story = { render: () => <Example mode="disabled" /> };
export const BeforeFirstRun: Story = {
  render: () => <Example mode="before-first-run" />,
};
export const Checking: Story = { render: () => <Example mode="checking" /> };
export const NoPublishedFeed: Story = {
  render: () => <Example mode="unavailable" />,
};
export const Current: Story = { render: () => <Example mode="current" /> };
export const Available: Story = { render: () => <Example /> };
export const NetworkError: Story = { render: () => <Example mode="error" /> };
export const Portable: Story = { render: () => <Example mode="portable" /> };
export const Banner: Story = { render: () => <Example mode="banner" /> };
export const Confirmation: Story = {
  render: () => <Example />,
  play: async () => {
    const body = within(document.body);
    await userEvent.click(
      await body.findByRole('button', { name: 'Review update' }),
    );
    await expect(body.getByRole('dialog')).toHaveTextContent(
      'installer is unsigned',
    );
  },
};
export const IntegrityError: Story = {
  render: () => <Example mode="integrity-error" />,
  play: async () => {
    const body = within(document.body);
    await userEvent.click(
      await body.findByRole('button', { name: 'Review update' }),
    );
    await userEvent.click(
      body.getByRole('button', { name: 'Download and install' }),
    );
    await expect(
      await within(body.getByRole('dialog')).findByRole('alert'),
    ).toHaveTextContent('Nothing was installed');
  },
};
export const Downloading: Story = {
  render: () => <Example mode="progress" />,
  play: async () => {
    const body = within(document.body);
    await userEvent.click(
      await body.findByRole('button', { name: 'Review update' }),
    );
    await userEvent.click(
      body.getByRole('button', { name: 'Download and install' }),
    );
    await expect(
      await within(body.getByRole('dialog')).findByRole('status'),
    ).toHaveTextContent('Downloaded 2.0 MiB');
  },
};
export const Dark: Story = {
  parameters: { themes: { themeOverride: 'dark' } },
  render: () => <Example dark />,
};
