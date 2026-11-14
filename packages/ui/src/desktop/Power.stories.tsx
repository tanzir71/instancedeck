import { useState } from 'react';
import type { Meta, StoryObj } from '@storybook/react-vite';
import { expect, userEvent, within, waitFor } from 'storybook/test';
import { CustomAppEditor } from './CustomAppEditor';
import { ProfileTransfer } from './ProfileTransfer';
import { SharedHistory } from './SharedHistory';
import { DesktopApp, DesktopShell } from './DesktopApp';
import { Wizard } from './Wizard';
import { fixtureSnapshot, fixtureProfiles, fixtureApps } from './fixtures';
import { createPreviewClient } from './previewClient';
import type { CustomApp, DesktopClient } from './model';
const meta = {
  title: 'Desktop/Power features',
  tags: ['p5'],
  parameters: { layout: 'fullscreen' },
} satisfies Meta;
export default meta;
type Story = StoryObj<typeof meta>;
const initial: CustomApp = {
  id: 'custom-editor',
  name: 'My editor',
  executable: 'C:\\Official apps\\Editor.exe',
  args: ['--data={profile}'],
  env: {},
  matchArgument: '--data={profile}',
};
function Editor({
  input = initial,
  failure = false,
  invalid = false,
}: {
  input?: CustomApp;
  failure?: boolean;
  invalid?: boolean;
}) {
  const [saved, setSaved] = useState(false);
  const [client] = useState(() => {
    const client = createPreviewClient(fixtureSnapshot, true);
    return {
      ...client,
      ...(failure
        ? {
            addCustom: async () => {
              throw new Error('This custom app id already exists.');
            },
          }
        : {}),
      ...(invalid
        ? {
            previewCustom: async () => {
              throw new Error(
                'Match argument must be part of the launch arguments.',
              );
            },
          }
        : {}),
    };
  });
  return saved ? (
    <h1>Custom recipe saved in memory</h1>
  ) : (
    <CustomAppEditor
      client={client}
      initial={input}
      root={fixtureSnapshot.settings.profilesRoot}
      onClose={() => setSaved(true)}
      onSaved={async () => setSaved(true)}
    />
  );
}
export const CustomArguments: Story = { render: () => <Editor /> };
export const CustomEnvironment: Story = {
  render: () => (
    <Editor
      input={{
        ...initial,
        args: [],
        matchArgument: '',
        env: { APP_HOME: '{profile}\\home' },
      }}
    />
  ),
};
export const CustomInvalid: Story = { render: () => <Editor invalid /> };
export const CustomSaveError: Story = {
  render: () => <Editor failure />,
  play: async () => {
    const body = within(document.body);
    const button = await body.findByRole('button', { name: 'Save custom app' });
    await waitFor(() => expect(button).toBeEnabled());
    await userEvent.click(button);
    await expect(await body.findByRole('alert')).toHaveTextContent(
      'already exists',
    );
  },
};
function Transfer({ failure = false }: { failure?: boolean }) {
  const [client] = useState<DesktopClient>(() => ({
    ...createPreviewClient(fixtureSnapshot, true),
    pickImport: async () => ({
      document: JSON.stringify({ schemaVersion: 1, profiles: [] }),
      added: 2,
      skipped: 1,
    }),
    importProfiles: async () => {
      if (failure)
        throw new Error(
          'A profile hotkey conflicts with the global switcher. No metadata was changed.',
        );
      return { added: 2, skipped: 1 };
    },
    exportProfiles: async () => 'D:\\Export fixture\\profiles.json',
  }));
  return (
    <DesktopShell
      screen="settings"
      onNavigate={() => {}}
      onWindow={async () => {}}
    >
      <ProfileTransfer client={client} onImported={async () => {}} />
    </DesktopShell>
  );
}
export const Metadata: Story = { render: () => <Transfer /> };
export const ImportError: Story = { render: () => <Transfer failure /> };
function History({
  enabled = false,
  failure = false,
}: {
  enabled?: boolean;
  failure?: boolean;
}) {
  const [profile, setProfile] = useState(() => {
    const profile = fixtureProfiles.find((p) => p.appId === 'codex')!;
    return {
      ...profile,
      options: { ...profile.options, sharedCodexHistory: enabled },
    };
  });
  const [client] = useState(() => ({
    ...createPreviewClient(fixtureSnapshot, true),
    sharedHistory: async (_: string, value: boolean) => {
      if (failure)
        throw new Error(
          'The profile is still running. Stop it before changing sessions sharing.',
        );
      setProfile((p) => ({
        ...p,
        options: { ...p.options, sharedCodexHistory: value },
      }));
    },
  }));
  return (
    <DesktopShell screen="deck" onNavigate={() => {}} onWindow={async () => {}}>
      <SharedHistory
        client={client}
        profile={profile}
        onSaved={async () => {}}
      />
    </DesktopShell>
  );
}
export const PrivateHistory: Story = { render: () => <History /> };
export const SharedSessions: Story = { render: () => <History enabled /> };
export const SharedHistoryError: Story = {
  render: () => <History failure />,
  play: async () => {
    const body = within(document.body);
    await userEvent.click(
      body.getByRole('button', { name: 'Restore private history' }),
    );
    await expect(await body.findByRole('alert')).toHaveTextContent(
      'still running',
    );
  },
};
export const DarkDeck: Story = {
  parameters: { themes: { themeOverride: 'dark' } },
  render: () => (
    <DesktopApp
      client={createPreviewClient(
        {
          ...fixtureSnapshot,
          settings: { ...fixtureSnapshot.settings, theme: 'dark' },
        },
        true,
      )}
    />
  ),
};
export const DarkWizard: Story = {
  parameters: { themes: { themeOverride: 'dark' } },
  render: () => (
    <DesktopShell
      theme="dark"
      screen="deck"
      onNavigate={() => {}}
      onWindow={async () => {}}
    >
      <Wizard
        apps={fixtureApps}
        root={fixtureSnapshot.settings.profilesRoot}
        onClose={() => {}}
        onBrowse={async () => null}
        onCreate={async () => {}}
        initialStep={1}
        initialName="Work 日本語"
      />
    </DesktopShell>
  ),
};
export const DarkSettings: Story = {
  parameters: { themes: { themeOverride: 'dark' } },
  render: () => (
    <DesktopApp
      initialScreen="settings"
      client={createPreviewClient(
        {
          ...fixtureSnapshot,
          settings: { ...fixtureSnapshot.settings, theme: 'dark' },
        },
        true,
      )}
    />
  ),
};
