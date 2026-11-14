import type { Meta, StoryObj } from '@storybook/react-vite';
import { expect, userEvent, within } from 'storybook/test';
import { ButtonLink } from '../components/Button';
import {
  CTABand,
  Eyebrow,
  FeatureTriplet,
  Hero,
  HeroFan,
  InstanceCard,
  LogoTextStrip,
  MarketingFooter,
  SecurityBand,
  SiteHeader,
} from './LandingSections';

const meta = {
  title: 'Marketing/Sections',
  parameters: { layout: 'fullscreen' },
} satisfies Meta;
export default meta;
type Story = StoryObj<typeof meta>;
const desktop = { verifyViewport: { width: 1440, height: 900 } };
const laptop = { verifyViewport: { width: 1024, height: 768 } };
const tablet = { verifyViewport: { width: 768, height: 1024 } };
const mobile = { verifyViewport: { width: 390, height: 844 } };
export const HeaderDesktop: Story = {
  parameters: desktop,
  render: () => <SiteHeader />,
};
export const HeaderLaptop: Story = { ...HeaderDesktop, parameters: laptop };
export const HeaderTablet: Story = { ...HeaderDesktop, parameters: tablet };
export const HeaderMobile: Story = { ...HeaderDesktop, parameters: mobile };
export const MobileMenu: Story = {
  ...HeaderMobile,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement.ownerDocument.body);
    await userEvent.click(canvas.getByRole('button', { name: 'Menu' }));
    await expect(
      canvas.getByRole('dialog', { name: 'Site navigation' }),
    ).toBeVisible();
  },
};
export const MobileMenuKeyboard: Story = {
  ...HeaderMobile,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement.ownerDocument.body);
    canvas.getByRole('button', { name: 'Menu' }).focus();
    await userEvent.keyboard('{Enter}');
    const dialog = canvas.getByRole('dialog');
    await expect(dialog).toBeVisible();
    within(dialog).getByRole('button', { name: 'Close menu' }).focus();
    await userEvent.keyboard('{Enter}');
    await expect(dialog).not.toBeVisible();
  },
};
export const HeroDesktop: Story = {
  parameters: desktop,
  render: () => <Hero />,
};
export const HeroLaptop: Story = { ...HeroDesktop, parameters: laptop };
export const HeroTablet: Story = { ...HeroDesktop, parameters: tablet };
export const HeroMobile: Story = { ...HeroDesktop, parameters: mobile };
export const Fan: Story = {
  parameters: desktop,
  render: () => (
    <div style={{ display: 'flex', width: 680, maxWidth: '100%', height: 780 }}>
      <HeroFan />
    </div>
  ),
};
export const CardRunning: Story = {
  render: () => (
    <InstanceCard
      name="Work"
      app="Claude Desktop"
      letters="Cl"
      path="Example profile folder"
      color="deep"
      running
    />
  ),
};
export const CardIdle: Story = {
  render: () => (
    <InstanceCard
      name="Personal"
      app="Claude Desktop"
      letters="Cl"
      path="Example profile folder"
      color="profile-terracotta"
    />
  ),
};
export const WorksDesktop: Story = {
  parameters: desktop,
  render: () => <LogoTextStrip />,
};
export const WorksLaptop: Story = { ...WorksDesktop, parameters: laptop };
export const WorksTablet: Story = { ...WorksDesktop, parameters: tablet };
export const WorksMobile: Story = { ...WorksDesktop, parameters: mobile };
export const PillarsDesktop: Story = {
  parameters: desktop,
  render: () => <FeatureTriplet />,
};
export const PillarsLaptop: Story = { ...PillarsDesktop, parameters: laptop };
export const PillarsTablet: Story = { ...PillarsDesktop, parameters: tablet };
export const PillarsMobile: Story = { ...PillarsDesktop, parameters: mobile };
export const SecurityDesktop: Story = {
  parameters: desktop,
  render: () => <SecurityBand />,
};
export const SecurityLaptop: Story = { ...SecurityDesktop, parameters: laptop };
export const SecurityTablet: Story = { ...SecurityDesktop, parameters: tablet };
export const SecurityMobile: Story = { ...SecurityDesktop, parameters: mobile };
export const SecuritySigned: Story = {
  parameters: desktop,
  render: () => <SecurityBand signingEnabled />,
};
export const CtaDesktop: Story = {
  parameters: desktop,
  render: () => <CTABand />,
};
export const CtaLaptop: Story = { ...CtaDesktop, parameters: laptop };
export const CtaTablet: Story = { ...CtaDesktop, parameters: tablet };
export const CtaMobile: Story = { ...CtaDesktop, parameters: mobile };
export const FooterDesktop: Story = {
  parameters: desktop,
  render: () => <MarketingFooter />,
};
export const FooterLaptop: Story = { ...FooterDesktop, parameters: laptop };
export const FooterTablet: Story = { ...FooterDesktop, parameters: tablet };
export const FooterMobile: Story = { ...FooterDesktop, parameters: mobile };
export const EyebrowLight: Story = {
  render: () => <Eyebrow>Why InstanceDeck</Eyebrow>,
};
export const EyebrowDark: Story = {
  render: () => (
    <div style={{ background: 'var(--deep)', padding: 32 }}>
      <Eyebrow onDark>Security and privacy</Eyebrow>
    </div>
  ),
};
export const ActionLink: Story = {
  render: () => (
    <ButtonLink href="https://github.com/tanzir71/instancedeck">
      View source on GitHub
    </ButtonLink>
  ),
};
export const FoundationMobile: Story = {
  parameters: mobile,
  render: () => (
    <>
      <SiteHeader />
      <main>
        <Hero />
        <LogoTextStrip />
        <FeatureTriplet />
        <SecurityBand />
        <CTABand />
      </main>
      <MarketingFooter />
    </>
  ),
};
