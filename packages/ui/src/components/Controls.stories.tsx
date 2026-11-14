import { useState } from 'react';
import type { Meta, StoryObj } from '@storybook/react-vite';
import { Button } from './Button';
import * as Controls from './Controls';
import {
  AddProfileCard,
  AppGroupHeader,
  EmptyState,
  MainInstanceCard,
  MonogramTile,
  ProfileCard,
  StatusDot,
} from './ProfileCard';
import { fixtureApps, fixtureProfiles } from '../desktop/fixtures';

const meta = {
  title: 'Components/Controls',
  parameters: { layout: 'padded' },
} satisfies Meta;
export default meta;
type Story = StoryObj<typeof meta>;
export const IconButton: Story = {
  render: () => (
    <Controls.IconButton label="Close dialog">×</Controls.IconButton>
  ),
};
export const Input: Story = {
  render: () => (
    <Controls.Input
      label="Profile name"
      defaultValue="Work"
      hint="Names never change the data path."
    />
  ),
};
export const Select: Story = {
  render: () => (
    <Controls.Select label="Update channel">
      <option>Stable</option>
      <option>Beta</option>
    </Controls.Select>
  ),
};
export const Switch: Story = {
  render: function Example() {
    const [checked, setChecked] = useState(true);
    return (
      <Controls.Switch
        label="Isolate Claude Code config (experimental)"
        checked={checked}
        onChange={setChecked}
      />
    );
  },
};
export const Checkbox: Story = {
  render: () => <Controls.Checkbox label="Desktop shortcut" defaultChecked />,
};
export const ColorSwatchPicker: Story = {
  render: function Example() {
    const [color, setColor] = useState<Profile['color']>('teal');
    return <Controls.ColorSwatchPicker value={color} onChange={setColor} />;
  },
};
export const Tooltip: Story = {
  render: () => (
    <Controls.Tooltip label="A profile keeps its own app data.">
      Profile isolation
    </Controls.Tooltip>
  ),
};
export const Dialog: Story = {
  render: () => (
    <Controls.Dialog title="Profile action" onClose={() => {}}>
      <div style={{ padding: 32 }}>
        <h2>Profile action</h2>
        <Button>Continue</Button>
      </div>
    </Controls.Dialog>
  ),
};
export const Sheet: Story = {
  render: () => (
    <Controls.Sheet title="Profile details" onClose={() => {}}>
      <Controls.Input label="Profile name" defaultValue="Work" />
    </Controls.Sheet>
  ),
};
export const Toast: Story = {
  render: () => <Controls.Toast message="Profile created" onClose={() => {}} />,
};
export const Tabs: Story = {
  render: function Example() {
    const [tab, setTab] = useState('environment');
    return (
      <Controls.Tabs
        items={[
          { id: 'environment', label: 'Environment' },
          { id: 'logs', label: 'Launch events' },
        ]}
        value={tab}
        onChange={setTab}
      />
    );
  },
};
export const Menu: Story = {
  render: () => (
    <Controls.Menu
      label="Profile actions"
      items={[
        { label: 'Open folder', onClick: () => {} },
        { label: 'Remove from deck', onClick: () => {} },
      ]}
    />
  ),
};
export const Kbd: Story = { render: () => <Controls.Kbd>Ctrl</Controls.Kbd> };
export const Badge: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: 16 }}>
      {(['verified', 'community', 'experimental'] as const).map((status) => (
        <Controls.Badge key={status} status={status} />
      ))}
    </div>
  ),
};
export const Stepper: Story = { render: () => <Controls.Stepper step={1} /> };
export const KeyValueList: Story = {
  render: () => (
    <Controls.KeyValueList
      values={{
        Version: '5.11492.5.0',
        Path: 'D:\\Profiles\\Zoë 日本語 with spaces',
      }}
    />
  ),
};
export const LogTable: Story = {
  render: () => (
    <Controls.LogTable
      rows={[
        {
          timestamp: '2026-11-14T08:00:00Z',
          fields: {
            recipeId: 'codex',
            strategy: 'direct',
            exitStatus: 'running',
          },
        },
      ]}
    />
  ),
};
export const EmptyLogTable: Story = {
  render: () => <Controls.LogTable rows={[]} />,
};
export const Monograms: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: 16 }}>
      <MonogramTile letters="Cl" tone="neutral" size="S" />
      <MonogramTile letters="Cx" />
      <MonogramTile letters="VS" tone="ink" size="L" />
    </div>
  ),
};
export const StatusDots: Story = {
  render: () => (
    <div>
      <StatusDot running />
      <StatusDot running={false} />
    </div>
  ),
};
export const ProfileRunning: Story = {
  render: () => (
    <div style={{ width: 224 }}>
      <ProfileCard profile={fixtureProfiles[0]!} running />
    </div>
  ),
};
export const ProfileIdle: Story = {
  render: () => (
    <div style={{ width: 224 }}>
      <ProfileCard profile={fixtureProfiles[2]!} />
    </div>
  ),
};
export const ProfileBusy: Story = {
  render: () => (
    <div style={{ width: 224 }}>
      <ProfileCard profile={fixtureProfiles[0]!} busy />
    </div>
  ),
};
export const ProfileLongName: Story = {
  render: () => (
    <div style={{ width: 224 }}>
      <ProfileCard
        profile={{
          ...fixtureProfiles[0]!,
          name: 'Client — A long international organisation name',
        }}
        running
      />
    </div>
  ),
};
export const MainRunning: Story = {
  render: () => (
    <div style={{ width: 224 }}>
      <MainInstanceCard appId="claude-desktop" running onLaunch={() => {}} />
    </div>
  ),
};
export const MainIdle: Story = {
  render: () => (
    <div style={{ width: 224 }}>
      <MainInstanceCard appId="codex" running={false} onLaunch={() => {}} />
    </div>
  ),
};
export const AddProfile: Story = {
  render: () => (
    <div style={{ width: 224 }}>
      <AddProfileCard onClick={() => {}} />
    </div>
  ),
};
export const GroupHeader: Story = {
  render: () => (
    <AppGroupHeader app={fixtureApps[0]!} onHealthCheck={() => {}} />
  ),
};
export const Empty: Story = {
  render: () => (
    <EmptyState
      title="Add your first profile"
      body="Keep work, personal and client accounts in their own space."
      action={{ label: 'New profile', onClick: () => {} }}
    />
  ),
};
type Profile = import('../desktop/model').Profile;
