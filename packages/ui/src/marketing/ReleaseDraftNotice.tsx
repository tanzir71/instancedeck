import copy from '../copy/en.json';
import styles from './Documentation.module.css';
export function ReleaseDraftNotice({
  tag,
  createdAt,
  runUrl,
  notes,
  showNotes = false,
}: {
  tag: string;
  createdAt: string;
  runUrl: string;
  notes: string;
  showNotes?: boolean;
}) {
  return (
    <section className={styles.draftNotice}>
      <h2>{copy.releaseDraft.title.replace('{tag}', tag)}</h2>
      <p>{copy.releaseDraft.body}</p>
      <p>
        {copy.releaseDraft.created.replace('{date}', createdAt.slice(0, 10))}
      </p>
      <p>
        <a href={`https://github.com/tanzir71/instancedeck/tree/${tag}`}>
          {copy.releaseDraft.source}
        </a>{' '}
        · <a href={runUrl}>{copy.releaseDraft.build}</a>
      </p>
      {showNotes && <pre>{notes}</pre>}
    </section>
  );
}
