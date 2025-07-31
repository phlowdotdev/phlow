import React from 'react';
import { useLatestRelease } from '@site/src/hooks/useLatestRelease';
import styles from './styles.module.css';

interface VersionBadgeProps {
  className?: string;
}

export default function VersionBadge({ className }: VersionBadgeProps): JSX.Element {
  const { version, loading, error, releaseUrl } = useLatestRelease();

  if (loading) {
    return (
      <div className={`${styles.versionContainer} ${className || ''}`}>
        <span className="badge badge--secondary">Loading...</span>
        <span className={styles.versionText}>Fetching version</span>
      </div>
    );
  }

  if (error) {
    console.warn('Failed to fetch latest version:', error);
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
      <span className={styles.versionText}>Latest Release</span>
    </div>
  );
}
