import React from 'react';
import { useLatestRelease } from '@site/src/hooks/useLatestRelease';
import styles from './styles.module.css';

interface VersionBadgeProps {
  className?: string;
}

export default function VersionBadge({ className }: VersionBadgeProps): JSX.Element {
  const { version, loading, releaseUrl } = useLatestRelease();

  if (loading) {
    return null;
  }


  const displayVersion = version || 'v0.0.42';
  const targetUrl = releaseUrl || 'https://github.com/phlowdotdev/phlow/releases';

  return (
    <div className={`${styles.versionContainer} ${className || ''}`}>
      <a
        href={targetUrl}
        target="_blank"
        rel="noopener noreferrer"
        className={styles.versionLink}
      >
        <span className="badge badge--primary">{displayVersion}</span>
      </a>
    </div>
  );
}
