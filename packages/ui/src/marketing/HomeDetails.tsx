import { type ReactNode } from 'react';
import copy from '../copy/en.json';
import { Eyebrow, siteLink } from './LandingSections';
import styles from './HomeDetails.module.css';

const text = copy.marketing;
export const faqItems = text.faqItems;

export function StepList() {
  return (
    <ol className={styles.steps}>
      {text.steps.map((step, index) => (
        <li key={step.title}>
          <span aria-hidden="true">{String(index + 1).padStart(2, '0')}</span>
          <h3>{step.title}</h3>
          <p>{step.body}</p>
        </li>
      ))}
    </ol>
  );
}

export function HowItWorks({ productShot }: { productShot: ReactNode }) {
  return (
    <section className={styles.how} aria-labelledby="how-title">
      <div className={styles.intro}>
        <Eyebrow>{text.how}</Eyebrow>
        <h2 id="how-title">{text.howTitle}</h2>
      </div>
      <StepList />
      <div className={styles.productField}>{productShot}</div>
    </section>
  );
}

export function UseCaseGrid() {
  return (
    <section className={styles.useCases} aria-labelledby="use-cases-title">
      <div className={styles.intro}>
        <Eyebrow>{text.useCases}</Eyebrow>
        <h2 id="use-cases-title">{text.useCasesTitle}</h2>
        <p>{text.useCasesIntro}</p>
      </div>
      <div className={styles.useCaseItems}>
        {text.useCaseItems.map((item) => (
          <article key={item.title}>
            <span
              style={{ background: `var(--${item.color})` }}
              aria-hidden="true"
            >
              {item.letters}
            </span>
            <h3>{item.title}</h3>
            <p>{item.body}</p>
          </article>
        ))}
      </div>
    </section>
  );
}

export function ClaudeAccountVisual() {
  return (
    <div
      className={styles.claudeVisual}
      role="img"
      aria-label={text.claudeVisualLabel}
    >
      {['work@company.com', 'me@acme-client.com'].map((account) => (
        <div className={styles.claudeWindow} key={account} aria-hidden="true">
          <div className={styles.windowBar}>
            <span>Claude · {account}</span>
            <span>– □ ×</span>
          </div>
          <div className={styles.windowBody}>
            <span />
            <span />
            <span />
            <span />
            <p>{text.signedInAs.replace('{account}', account)}</p>
          </div>
        </div>
      ))}
    </div>
  );
}

export function CodexHomeVisual() {
  return (
    <div
      className={styles.codexVisual}
      role="img"
      aria-label={text.codexVisualLabel}
    >
      {['acme', 'qa'].map((name) => (
        <pre
          key={name}
          aria-hidden="true"
        >{`Profiles\\codex\\${name}\\\n  codex-home\\\n    auth.json\n    sessions\\\n    config.toml\n  electron\\`}</pre>
      ))}
    </div>
  );
}

export function AppSpotlights({ linkPrefix = './' }: { linkPrefix?: string }) {
  return (
    <section className={styles.spotlights} aria-label={text.apps}>
      <article>
        <Eyebrow>Claude Desktop</Eyebrow>
        <h2>{text.claudeSpotlightTitle}</h2>
        <p>{text.claudeSpotlightBody}</p>
        <a href={siteLink(linkPrefix, 'claude-desktop-multiple-accounts/')}>
          {text.claudeGuideLink}
        </a>
        <ClaudeAccountVisual />
      </article>
      <article>
        <Eyebrow>Codex</Eyebrow>
        <h2>{text.codexSpotlightTitle}</h2>
        <p>{text.codexSpotlightBody}</p>
        <a href={siteLink(linkPrefix, 'codex-multiple-accounts/')}>
          {text.codexGuideLink}
        </a>
        <CodexHomeVisual />
      </article>
    </section>
  );
}

export function FAQItem({
  question,
  children,
  open = false,
}: {
  question: string;
  children: ReactNode;
  open?: boolean;
}) {
  return (
    <details className={styles.faqItem} open={open}>
      <summary>
        <span>{question}</span>
        <span className={styles.faqIndicator} aria-hidden="true" />
      </summary>
      <div>{children}</div>
    </details>
  );
}

export function FAQPreview({ linkPrefix = './' }: { linkPrefix?: string }) {
  return (
    <section className={styles.faq} aria-labelledby="faq-title">
      <div className={styles.intro}>
        <Eyebrow>{text.faq}</Eyebrow>
        <h2 id="faq-title">{text.faqTitle}</h2>
        <p>{text.faqIntro}</p>
        <a href={siteLink(linkPrefix, 'faq/')}>{text.allQuestions}</a>
      </div>
      <div>
        {faqItems
          .filter((item) => !['free', 'apps', 'mac'].includes(item.id))
          .map((item, index) => (
            <FAQItem key={item.id} question={item.question} open={index === 0}>
              <p>{item.answer}</p>
            </FAQItem>
          ))}
      </div>
    </section>
  );
}
