import { useEffect, useRef } from 'react';

const OceanBackground = () => {
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

    const waves = [
      { amplitude: 20, wavelength: 0.020, speed: 0.01, offset: 0, color: 'rgba(0, 150, 255, 0.4)' },
      { amplitude: 20, wavelength: 0.015, speed: 0.02, offset: 0, color: 'rgba(0, 150, 255, 0.4)' },
      { amplitude: 30, wavelength: 0.010, speed: 0.015, offset: 100, color: 'rgba(0, 120, 255, 0.3)' },
      { amplitude: 40, wavelength: 0.007, speed: 0.010, offset: 200, color: 'rgba(0, 100, 255, 0.2)' },
    ];

    const resizeCanvas = () => {
      width = window.innerWidth;
      height = window.innerHeight;
      canvas.width = width;
      canvas.height = height;
    };

    const animate = () => {
      ctx.clearRect(0, 0, width, height);

      waves.forEach((wave) => {
        ctx.beginPath();
        ctx.moveTo(0, height / 2);

        for (let x = 0; x <= width; x++) {
          ctx.lineTo(
            x,
            height / 2 + Math.sin(x * wave.wavelength + wave.offset) * wave.amplitude
          );
        }

        ctx.lineTo(width, height);
        ctx.lineTo(0, height);
        ctx.closePath();
        ctx.fillStyle = wave.color;
        ctx.fill();

        wave.offset += wave.speed;
      });

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
        width: '100%',
        height: '100%',
        zIndex: -1,
      }}
    />
  );
};

export default OceanBackground;
