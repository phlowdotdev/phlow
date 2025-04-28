import { useEffect, useRef } from 'react';

const WavesBackground = () => {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        let animationFrameId: number;
        let width = window.innerWidth;
        let height = window.innerHeight;
        canvas.width = width;
        canvas.height = height;

        let wave = {
            y: height / 2,
            length: 0.01,
            amplitude: 100,
            frequency: 0.02,
        };

        let increment = wave.frequency;

        const resizeCanvas = () => {
            width = window.innerWidth;
            height = window.innerHeight;
            canvas.width = width;
            canvas.height = height;
        };

        const animate = () => {
            ctx.clearRect(0, 0, width, height);

            ctx.beginPath();
            ctx.moveTo(0, height / 2);

            for (let i = 0; i < width; i++) {
                ctx.lineTo(i, wave.y + Math.sin(i * wave.length + increment) * wave.amplitude);
            }

            ctx.strokeStyle = 'rgba(0, 150, 255, 0.5)';
            ctx.lineWidth = 2;
            ctx.stroke();

            increment += wave.frequency;
            animationFrameId = requestAnimationFrame(animate);
        };

        window.addEventListener('resize', resizeCanvas);
        animate();

        return () => {
            window.removeEventListener('resize', resizeCanvas);
            cancelAnimationFrame(animationFrameId);
        };
    }, []);

    return (
        <canvas
            ref={canvasRef}
            style={{
                position: 'fixed',
                top: 0,
                left: 0,
                zIndex: -1,
                width: '100%',
                height: '100%',
            }}
        />
    );
};

export default WavesBackground;
