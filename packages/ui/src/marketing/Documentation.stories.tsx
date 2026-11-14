import type { Meta, StoryObj } from '@storybook/react-vite';
import {
  AppDirectoryCard,
  Breadcrumbs,
  ComparisonTable,
  DocLayout,
} from './Documentation';

const meta = {
  title: 'Marketing/Documentation',
  parameters: { layout: 'fullscreen' },
} satisfies Meta;
export default meta;
type Story = StoryObj<typeof meta>;
const desktop = { verifyViewport: { width: 1440, height: 900 } };
const laptop = { verifyViewport: { width: 1024, height: 768 } };
const tablet = { verifyViewport: { width: 768, height: 1024 } };
const mobile = { verifyViewport: { width: 390, height: 844 } };

export const Breadcrumb: Story = {
  render: () => (
    <Breadcrumbs
      items={[{ label: 'Apps', href: './apps/' }, { label: 'Codex' }]}
    />
  ),
};
export const DocumentDesktop: Story = {
  parameters: desktop,
  render: () => (
    <DocLayout
      title="How InstanceDeck isolates your apps"
      eyebrow="How it works"
      intro="Separate data folders and launch settings give each profile its own state."
      breadcrumbs={[{ label: 'How it works' }]}
    >
      <h2>One profile, one data folder</h2>
      <p>
        InstanceDeck launches the official app with its own data path. It never
        patches or repackages the app.
      </p>
      <pre>
        <code>
          {
            'CODEX_HOME=<profile>\\codex-home\nCODEX_ELECTRON_USER_DATA_PATH=<profile>\\electron'
          }
        </code>
      </pre>
      <aside>
        Profiles are isolation, not a sandbox. Processes run as your Windows
        user.
      </aside>
      <h2>Keep every account separate</h2>
      <p>
        Check the visible account in each profile after an app update.{' '}
        <a href="./apps/">See the compatibility directory.</a>
      </p>
    </DocLayout>
  ),
};
export const DocumentLaptop: Story = { ...DocumentDesktop, parameters: laptop };
export const DocumentTablet: Story = { ...DocumentDesktop, parameters: tablet };
export const DocumentMobile: Story = { ...DocumentDesktop, parameters: mobile };
export const DocumentRtl: Story = {
  ...DocumentMobile,
  decorators: [
    (Story) => (
      <div dir="rtl">
        <Story />
      </div>
    ),
  ],
};
export const TableDesktop: Story = {
  parameters: desktop,
  render: () => (
    <ComparisonTable
      caption="Example recipe mechanisms"
      columns={['Claude Desktop', 'Codex']}
      rows={[
        {
          label: 'App data',
          values: [
            'Separate LOCALAPPDATA and user-data directory',
            'Separate Electron data path',
          ],
        },
        {
          label: 'Sign-in',
          values: ['Guided snapshot', 'Inside each profile'],
        },
      ]}
    />
  ),
};
export const TableLaptop: Story = { ...TableDesktop, parameters: laptop };
export const TableTablet: Story = { ...TableDesktop, parameters: tablet };
export const TableMobile: Story = { ...TableDesktop, parameters: mobile };
export const AppCommunity: Story = {
  render: () => (
    <AppDirectoryCard
      name="Codex"
      letters="Cx"
      status="community"
      href="./apps/codex/"
      intro="Separate CODEX_HOME and Electron data paths isolate each profile."
      note="Community report; account identity requires a manual check."
    />
  ),
};
export const AppVerified: Story = {
  render: () => (
    <AppDirectoryCard
      name="Example verified recipe"
      letters="Ex"
      status="verified"
      href="#"
      intro="Storybook fixture demonstrating the Verified badge."
    />
  ),
};
export const AppExperimental: Story = {
  render: () => (
    <AppDirectoryCard
      name="Example experimental recipe"
      letters="Ex"
      status="experimental"
      href="#"
      intro="Storybook fixture demonstrating an experimental recipe and its limitations."
    />
  ),
};
export const AppLongName: Story = {
  parameters: mobile,
  render: () => (
    <AppDirectoryCard
      name="An example application with a very long display name"
      letters="Ex"
      status="community"
      href="#"
      intro="This fixture checks wrapping at the narrowest supported width."
    />
  ),
};
