import { useState, useEffect } from 'react';

interface GitHubRelease {
  tag_name: string;
  name: string;
  published_at: string;
  html_url: string;
}

interface UseLatestReleaseReturn {
  version: string | null;
  loading: boolean;
  error: string | null;
  releaseUrl: string | null;
}

export function useLatestRelease(
  owner: string = 'phlowdotdev',
  repo: string = 'phlow'
): UseLatestReleaseReturn {
  const [version, setVersion] = useState<string | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [releaseUrl, setReleaseUrl] = useState<string | null>(null);

  useEffect(() => {
    const fetchLatestRelease = async () => {
      try {
        setLoading(true);
        setError(null);

        const response = await fetch(
          `https://api.github.com/repos/${owner}/${repo}/releases/latest`,
          {
            headers: {
              'Accept': 'application/vnd.github.v3+json',
            },
          }
        );

        if (!response.ok) {
          throw new Error(`GitHub API error: ${response.status}`);
        }

        const release: GitHubRelease = await response.json();
        setVersion(release.tag_name);
        setReleaseUrl(release.html_url);
      } catch (err) {
        console.error('Failed to fetch latest release:', err);
        setError(err instanceof Error ? err.message : 'Unknown error');
        // Fallback to hardcoded version if API fails
        setVersion('v0.0.42');
        setReleaseUrl('https://github.com/phlowdotdev/phlow/releases');
      } finally {
        setLoading(false);
      }
    };

    fetchLatestRelease();
  }, [owner, repo]);

  return { version, loading, error, releaseUrl };
}
