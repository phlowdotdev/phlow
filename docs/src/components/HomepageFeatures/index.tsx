import type { ReactNode } from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Low Code, Maximum Power',
    description: (
      <>
        Build APIs, automations, and workflows with simple YAML — no heavy coding required.
      </>
    ),
  },
  {
    title: 'Scalable by Design',
    description: (
      <>
        Powered by Rust, Phlow runs with extreme speed, low resource usage, and scales effortlessly across any environment.
      </>
    ),
  },
  {
    title: 'Fully Observable',
    description: (
      <>
        Native OpenTelemetry integration lets you trace, measure, and monitor every flow in real time.
      </>
    ),
  },
];

function Feature({ title, description }: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
