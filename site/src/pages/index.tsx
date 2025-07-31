import type { ReactNode } from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Heading from '@theme/Heading';
import ClockSvg from '@site/static/img/clock.svg';
import CodespaceSvg from '@site/static/img/codespace.svg';
import styles from './index.module.css';
import OceanBackground from '../components/Backgrounds/OceanBackground';
import { HomeButton } from '../components/Buttons';
import VersionBadge from '@site/src/components/VersionBadge';

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx('', styles.heroBanner)}>
      <div className="container">
        <div className="hero__image">
          <img
            src="/img/logo.svg"
            alt="Phlow Logo"
            className="hero__logo"
          />
        </div>
        <Heading as="h1" className="hero__title">
          {siteConfig.title}
        </Heading>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <VersionBadge />
        <div className='buttons--home'>
          <HomeButton
            className="button button--secondary button--lg button--start-tutorial"
            to="/docs/intro"
          >Phlow Tutorial - 5min <ClockSvg /></HomeButton>
          <HomeButton
            className="button button--secondary button--lg button--start-codespace"
            target='_blank'
            to="https://github.com/codespaces/new?repo=phlowdotdev/phlow-mirror-request">
            Run now in codespace <CodespaceSvg /></HomeButton>
        </div>
      </div>
    </header>
  );
}

export default function Home(): ReactNode {
  return (
    <Layout
      title={`Build Flows, Not Code`}
      description="Phlow lets you launch scalable, modular backends in record time — no complex coding, just pure flow.">
      <HomepageHeader />
      <main>
        <HomepageFeatures />
      </main>
      <OceanBackground />
    </Layout>
  );
}
