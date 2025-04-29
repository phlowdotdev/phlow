import type { ReactNode } from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Heading from '@theme/Heading';
import ClockSvg from '@site/static/img/clock.svg';
import styles from './index.module.css';
import OceanBackground from '../components/Backgrounds/OceanBackground';

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
        <div className={styles.buttons}>
          <Link
            className="button button--secondary button--lg button--start-tutorial"
            to="/docs/intro">
            Phlow Tutorial - 5min <ClockSvg />
          </Link>
        </div>
      </div>
    </header>
  );
}

export default function Home(): ReactNode {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={`Build Flows, Not Code`}
      description="Phlow lets you launch scalable, modular backends in record time — no complex coding, just pure flow.">
      <OceanBackground />
      <HomepageHeader />
      <main>
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
