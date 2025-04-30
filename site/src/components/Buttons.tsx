import React, { useState } from 'react';
import { motion } from 'framer-motion';
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
            <motion.div
                whileTap={{ scale: 0.98 }}
                style={{ display: 'inline-block' }}
            >
                <Link
                    className={`button ${className}`}
                    target={target}
                    to={to}
                    onMouseDown={handlePressIn}
                    onMouseUp={handlePressOut}
                    onMouseLeave={handlePressOut}
                >
                    {children}
                </Link>
            </motion.div>
        </div>
    );
};
