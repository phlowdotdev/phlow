import React, { useState } from 'react';
import { spring, Motion } from 'react-motion';
import styles from '../pages/index.module.css';
import Link from '@docusaurus/Link';

interface HomeButtonProps {
    className?: string;
    target?: string;
    to: string;
    children: React.ReactNode;
}

export const HomeButton: React.FC<HomeButtonProps> = ({ className, target, to, children }) => {
    const [pressed, setPressed] = useState(false);

    const handlePressIn = () => setPressed(true);
    const handlePressOut = () => setPressed(false);

    return (
        <div className={styles.buttons}>
            <Motion style={{ scale: spring(pressed ? 0.98 : 1, { stiffness: 300, damping: 20 }) }}>
                {({ scale }) => (
                    <Link
                        className={`button ${className}`}
                        target={target}
                        to={to}
                        style={{ transform: `scale(${scale})` }}
                        onMouseDown={handlePressIn}
                        onMouseUp={handlePressOut}
                        onMouseLeave={handlePressOut}
                    >
                        {children}
                    </Link>
                )}
            </Motion>
        </div>
    );
};
