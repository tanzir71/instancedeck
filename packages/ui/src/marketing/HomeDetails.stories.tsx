import type { Meta, StoryObj } from '@storybook/react-vite';
import { expect, userEvent, within } from 'storybook/test';
import productShot from '../../../../design/20-app-deck@2x.png';
import copy from '../copy/en.json';
import {
  AppSpotlights,
  ClaudeAccountVisual,
  CodexHomeVisual,
  FAQItem,
  FAQPreview,
  HowItWorks,
  StepList,
  UseCaseGrid,
} from './HomeDetails';
import {
  CTABand,
  FeatureTriplet,
  Hero,
  LogoTextStrip,
  MarketingFooter,
  SecurityBand,
  SiteHeader,
} from './LandingSections';

const meta = {
  title: 'Marketing/Details',
  parameters: { layout: 'fullscreen' },
} satisfies Meta;
export default meta;
type Story = StoryObj<typeof meta>;
const desktop = { verifyViewport: { width: 1440, height: 900 } };
const laptop = { verifyViewport: { width: 1024, height: 768 } };
const tablet = { verifyViewport: { width: 768, height: 1024 } };
const mobile = { verifyViewport: { width: 390, height: 844 } };
const shot = (
  <img
    src={productShot}
    width={2240}
    height={1400}
    alt={copy.marketing.productShotAlt}
  />
);

export const HowDesktop: Story = {
  parameters: desktop,
  render: () => <HowItWorks productShot={shot} />,
};
export const HowLaptop: Story = { ...HowDesktop, parameters: laptop };
export const HowTablet: Story = { ...HowDesktop, parameters: tablet };
export const Steps: Story = { render: () => <StepList /> };
export const UseCasesDesktop: Story = {
  parameters: desktop,
  render: () => <UseCaseGrid />,
};
export const UseCasesLaptop: Story = { ...UseCasesDesktop, parameters: laptop };
export const UseCasesTablet: Story = { ...UseCasesDesktop, parameters: tablet };
export const SpotlightDesktop: Story = {
  parameters: desktop,
  render: () => <AppSpotlights />,
};
export const SpotlightLaptop: Story = {
  ...SpotlightDesktop,
  parameters: laptop,
};
export const SpotlightTablet: Story = {
  ...SpotlightDesktop,
  parameters: tablet,
};
export const ClaudeVisual: Story = { render: () => <ClaudeAccountVisual /> };
export const CodexVisual: Story = { render: () => <CodexHomeVisual /> };
export const FaqDesktop: Story = {
  parameters: desktop,
  render: () => <FAQPreview />,
};
export const FaqLaptop: Story = { ...FaqDesktop, parameters: laptop };
export const FaqTablet: Story = { ...FaqDesktop, parameters: tablet };
export const FaqClosed: Story = {
  render: () => (
    <FAQItem question={copy.marketing.faqItems[1]!.question}>
      <p>{copy.marketing.faqItems[1]!.answer}</p>
    </FAQItem>
  ),
};
export const FaqOpen: Story = {
  render: () => (
    <FAQItem question={copy.marketing.faqItems[1]!.question} open>
      <p>{copy.marketing.faqItems[1]!.answer}</p>
    </FAQItem>
  ),
};
export const FaqInteraction: Story = {
  ...FaqClosed,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const summary = canvas
      .getByText(copy.marketing.faqItems[1]!.question)
      .closest('summary')!;
    summary.focus();
    await userEvent.click(summary);
    await expect(
      canvas.getByText(copy.marketing.faqItems[1]!.answer),
    ).toBeVisible();
    await userEvent.click(summary);
    await expect(
      canvas.getByText(copy.marketing.faqItems[1]!.answer),
    ).not.toBeVisible();
  },
};
export const HomeDesktop: Story = {
  parameters: desktop,
  render: () => (
    <>
      <SiteHeader />
      <main>
        <Hero />
        <LogoTextStrip />
        <FeatureTriplet />
        <HowItWorks productShot={shot} />
        <UseCaseGrid />
        <AppSpotlights />
        <SecurityBand />
        <FAQPreview />
        <CTABand />
      </main>
      <MarketingFooter />
    </>
  ),
};
export const HomeLaptop: Story = { ...HomeDesktop, parameters: laptop };
export const HomeTablet: Story = { ...HomeDesktop, parameters: tablet };
export const HomeMobile: Story = { ...HomeDesktop, parameters: mobile };
