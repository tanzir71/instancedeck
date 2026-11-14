import { getStoryContext, type TestRunnerConfig } from '@storybook/test-runner';
import AxeBuilder from '@axe-core/playwright';

const config: TestRunnerConfig = {
  async preVisit(page, context) {
    const story = await getStoryContext(page, context);
    const viewport = story.parameters.verifyViewport;
    await page.setViewportSize(viewport ?? { width: 1280, height: 800 });
  },
  async postVisit(page) {
    const { violations } = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
      .analyze();
    if (violations.length)
      throw new Error(
        JSON.stringify(
          violations.map(({ id, nodes }) => ({
            id,
            targets: nodes.map((n) => n.target),
          })),
          null,
          2,
        ),
      );
  },
};
export default config;
