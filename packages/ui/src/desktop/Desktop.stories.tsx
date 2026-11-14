import { useState } from 'react';
import type { Meta, StoryObj } from '@storybook/react-vite';
import { expect, userEvent, within } from 'storybook/test';
import {
  DesktopApp,
  DesktopShell,
  DeckScreen,
  ConfirmProfileAction,
} from './DesktopApp';
import {
  AppsScreen,
  DiagnosticsScreen,
  ProfileDetail,
  SettingsScreen,
} from './ScreenViews';
import { SignInAssistant, RecoveryDialog } from './SignInAssistant';
import { Wizard } from './Wizard';
import { createPreviewClient, emptyPreviewSnapshot } from './previewClient';
import { fixtureApps, fixtureProfiles, fixtureSnapshot } from './fixtures';
import type { DesktopClient } from './model';

const meta = {
  title: 'Desktop/Screens',
  parameters: { layout: 'fullscreen' },
} satisfies Meta;
export default meta;
type Story = StoryObj<typeof meta>;
const noop = () => {};
const done = async () => {};
const browse = async () => null;
function ReferenceDeck({ children }: { children?: React.ReactNode }) {
  return (
    <DesktopShell screen="deck" onNavigate={noop} onWindow={done}>
      <DeckScreen
        snapshot={fixtureSnapshot}
        referenceTime={Date.parse('2026-11-14T12:00:00Z')}
        search=""
        onSearch={noop}
        onNew={noop}
        onProfileAction={noop}
        onDetails={noop}
        onMainAction={noop}
        onHealth={noop}
      />
      {children}
    </DesktopShell>
  );
}
export const Deck: Story = { render: () => <ReferenceDeck /> };
export const Interactive: Story = {
  render: function Example() {
    const [client] = useState(() => createPreviewClient(fixtureSnapshot, true));
    return <DesktopApp client={client} />;
  },
};
export const Empty: Story = {
  render: () => (
    <DesktopApp
      client={createPreviewClient({
        ...emptyPreviewSnapshot,
        apps: fixtureApps,
      })}
    />
  ),
};
export const Loading: Story = {
  render: () => (
    <DesktopApp
      client={{
        ...createPreviewClient(),
        snapshot: () => new Promise(() => {}),
      }}
    />
  ),
};
export const Error: Story = {
  render: () => (
    <DesktopApp
      client={{
        ...createPreviewClient(),
        snapshot: async () => {
          throw new globalThis.Error(
            'The profile metadata could not be opened. No files were changed.',
          );
        },
      }}
    />
  ),
};
export const LongNames: Story = {
  render: () => (
    <DesktopApp
      client={createPreviewClient({
        ...fixtureSnapshot,
        profiles: fixtureProfiles.map((profile) => ({
          ...profile,
          name: 'International client — Zoë 日本語 with a longer project name',
        })),
      })}
    />
  ),
};
export const Rtl: Story = {
  render: () => (
    <div dir="rtl">
      <ReferenceDeck />
    </div>
  ),
};
export const NoResults: Story = {
  render: () => (
    <DesktopShell screen="deck" onNavigate={noop} onWindow={done}>
      <DeckScreen
        snapshot={fixtureSnapshot}
        search="No matching name"
        onSearch={noop}
        onNew={noop}
        onProfileAction={noop}
        onDetails={noop}
        onMainAction={noop}
        onHealth={noop}
      />
    </DesktopShell>
  ),
};
export const StatusError: Story = {
  render: () => (
    <DesktopApp
      client={createPreviewClient({
        ...fixtureSnapshot,
        statusErrors: {
          work: 'The official app could not be resolved. Refresh Apps.',
        },
      })}
    />
  ),
};
export const WizardApp: Story = {
  render: () => (
    <ReferenceDeck>
      <Wizard
        apps={fixtureApps}
        root={'D:\\Profiles'}
        onClose={noop}
        onCreate={done}
        onBrowse={browse}
      />
    </ReferenceDeck>
  ),
};
export const WizardName: Story = {
  render: () => (
    <ReferenceDeck>
      <Wizard
        apps={fixtureApps}
        root={'D:\\Profiles'}
        initialId="01JC4Q00000000000000000000"
        initialAppId="claude-desktop"
        initialName="Client — Acme"
        initialColor="teal"
        initialStep={1}
        onClose={noop}
        onCreate={done}
        onBrowse={browse}
      />
    </ReferenceDeck>
  ),
};
export const WizardOptions: Story = {
  render: () => (
    <ReferenceDeck>
      <Wizard
        apps={fixtureApps}
        root={'D:\\Profiles'}
        initialName="Work"
        initialStep={2}
        onClose={noop}
        onCreate={done}
        onBrowse={browse}
      />
    </ReferenceDeck>
  ),
};
export const WizardMissingApp: Story = {
  render: () => (
    <ReferenceDeck>
      <Wizard
        apps={fixtureApps.map((app) => ({ ...app, installed: null }))}
        root={'D:\\Profiles'}
        onClose={noop}
        onCreate={done}
        onBrowse={browse}
      />
    </ReferenceDeck>
  ),
};
export const WizardError: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement.ownerDocument.body);
    await userEvent.click(
      canvas.getByRole('button', { name: 'Create profile' }),
    );
    await expect(canvas.getByRole('alert')).toHaveTextContent(
      'The selected drive is unavailable.',
    );
  },
  render: () => (
    <ReferenceDeck>
      <Wizard
        apps={fixtureApps}
        root={'D:\\Profiles'}
        initialName="Work"
        initialStep={2}
        onClose={noop}
        onCreate={async () => {
          throw new globalThis.Error('The selected drive is unavailable.');
        }}
        onBrowse={browse}
      />
    </ReferenceDeck>
  ),
};
export const SignIn: Story = {
  render: () => (
    <ReferenceDeck>
      <SignInAssistant
        name="Client — Acme"
        journalSaved
        onFinish={noop}
        onRestore={noop}
      />
    </ReferenceDeck>
  ),
};
export const SignInPreflight: Story = {
  render: () => (
    <ReferenceDeck>
      <SignInAssistant
        name="Work"
        step={0}
        journalSaved={false}
        busy
        onFinish={noop}
        onRestore={noop}
      />
    </ReferenceDeck>
  ),
};
export const SignInRestoring: Story = {
  render: () => (
    <ReferenceDeck>
      <SignInAssistant
        name="Work"
        step={3}
        journalSaved
        busy
        onFinish={noop}
        onRestore={noop}
      />
    </ReferenceDeck>
  ),
};
export const SignInError: Story = {
  render: () => (
    <ReferenceDeck>
      <SignInAssistant
        name="Work"
        step={2}
        journalSaved
        error="A file is still open. Restore the main Claude before retrying."
        onFinish={noop}
        onRestore={noop}
      />
    </ReferenceDeck>
  ),
};
export const Recovery: Story = {
  render: () => (
    <ReferenceDeck>
      <RecoveryDialog onRestore={noop} onResume={noop} />
    </ReferenceDeck>
  ),
};
export const RecoveryError: Story = {
  render: () => (
    <ReferenceDeck>
      <RecoveryDialog
        error="The main slot is occupied. Your stash is retained."
        onRestore={noop}
        onResume={noop}
      />
    </ReferenceDeck>
  ),
};
export const Profile: Story = {
  render: () => (
    <ReferenceDeck>
      <ProfileDetail
        profile={fixtureProfiles[0]!}
        client={createPreviewClient(fixtureSnapshot)}
        onClose={noop}
        onSaved={done}
        onAction={noop}
      />
    </ReferenceDeck>
  ),
};
export const Apps: Story = {
  render: () => (
    <DesktopShell screen="apps" onNavigate={noop} onWindow={done}>
      <AppsScreen
        snapshot={fixtureSnapshot}
        onRefresh={noop}
        onNewProfile={noop}
      />
    </DesktopShell>
  ),
};
export const AppsMissing: Story = {
  render: () => (
    <DesktopShell screen="apps" onNavigate={noop} onWindow={done}>
      <AppsScreen
        snapshot={{
          ...fixtureSnapshot,
          apps: fixtureApps.map((app) => ({
            ...app,
            installed: null,
            error: 'Not installed for this Windows user.',
          })),
        }}
        onRefresh={noop}
        onNewProfile={noop}
      />
    </DesktopShell>
  ),
};
export const Diagnostics: Story = {
  render: () => (
    <DesktopShell screen="diagnostics" onNavigate={noop} onWindow={done}>
      <DiagnosticsScreen
        client={createPreviewClient(fixtureSnapshot)}
        onCopied={noop}
      />
    </DesktopShell>
  ),
};
export const DiagnosticsLoading: Story = {
  render: () => (
    <DesktopShell screen="diagnostics" onNavigate={noop} onWindow={done}>
      <DiagnosticsScreen
        client={{
          ...createPreviewClient(),
          diagnostics: () => new Promise(() => {}),
        }}
        onCopied={noop}
      />
    </DesktopShell>
  ),
};
export const DiagnosticsError: Story = {
  render: () => (
    <DesktopShell screen="diagnostics" onNavigate={noop} onWindow={done}>
      <DiagnosticsScreen
        client={{
          ...createPreviewClient(),
          diagnostics: async () => {
            throw new globalThis.Error(
              'The diagnostics metadata is unavailable.',
            );
          },
        }}
        onCopied={noop}
      />
    </DesktopShell>
  ),
};
export const Settings: Story = {
  render: () => (
    <DesktopShell screen="settings" onNavigate={noop} onWindow={done}>
      <SettingsScreen
        settings={fixtureSnapshot.settings}
        onSave={done}
        onBrowse={browse}
      />
    </DesktopShell>
  ),
};
export const FirstRun: Story = {
  render: () => (
    <DesktopApp
      client={createPreviewClient({
        ...fixtureSnapshot,
        settings: { ...fixtureSnapshot.settings, onboardingCompleted: false },
      })}
    />
  ),
};
export const ConfirmForce: Story = {
  render: () => (
    <ReferenceDeck>
      <ConfirmProfileAction
        profile={fixtureProfiles[0]!}
        action="force-close"
        busy={false}
        onClose={noop}
        onConfirm={noop}
      />
    </ReferenceDeck>
  ),
};
export const ConfirmRemove: Story = {
  render: () => (
    <ReferenceDeck>
      <ConfirmProfileAction
        profile={fixtureProfiles[0]!}
        action="remove"
        busy={false}
        onClose={noop}
        onConfirm={noop}
      />
    </ReferenceDeck>
  ),
};
export const ConfirmRecycle: Story = {
  render: () => (
    <ReferenceDeck>
      <ConfirmProfileAction
        profile={fixtureProfiles[0]!}
        action="recycle"
        busy={false}
        onClose={noop}
        onConfirm={noop}
      />
    </ReferenceDeck>
  ),
};
export const ConfirmationError: Story = {
  render: () => (
    <ReferenceDeck>
      <ConfirmProfileAction
        profile={fixtureProfiles[0]!}
        action="recycle"
        busy={false}
        error="Close the profile before moving its data."
        onClose={noop}
        onConfirm={noop}
      />
    </ReferenceDeck>
  ),
};
export const ConfirmationBusy: Story = {
  render: () => (
    <ReferenceDeck>
      <ConfirmProfileAction
        profile={fixtureProfiles[0]!}
        action="recycle"
        busy
        onClose={noop}
        onConfirm={noop}
      />
    </ReferenceDeck>
  ),
};
export const EventError: Story = {
  render: () => {
    const client: DesktopClient = {
      ...createPreviewClient(fixtureSnapshot),
      subscribe: async (_, onError) => {
        onError('Status updates paused. Refresh to retry.');
        return noop;
      },
    };
    return <DesktopApp client={client} />;
  },
};
