import React, { useEffect, useRef } from 'react';
import Link from '@docusaurus/Link';
import styles from './styles.module.css';

interface FloatingLetter {
  char: string;
  class: string;
  count: number;
}

interface Position {
  top: string;
  left: string;
}

interface LetterPosition {
  char: string;
  x: number;
  y: number;
  originalChar: string;
}

export default function AnimatedLanding(): JSX.Element {
  const logoContainerRef = useRef<HTMLDivElement>(null);
  const floatingLettersRef = useRef<HTMLDivElement>(null);
  const asciiLogoRef = useRef<HTMLPreElement>(null);

  const isMobile = () => {
    return typeof window !== 'undefined' && window.innerWidth <= 768;
  };

  const createBackgroundFloatingLetters = () => {
    if (!floatingLettersRef.current) return;
    
    const container = floatingLettersRef.current;
    const letters: FloatingLetter[] = [
      { char: 'd', class: 'd', count: isMobile() ? 1 : 2 },
      { char: 'e', class: 'e', count: isMobile() ? 1 : 2 },
      { char: 'x', class: 'x', count: 1 },
      { char: 's', class: 's', count: isMobile() ? 0 : 1 },
      { char: 'u', class: 'u', count: isMobile() ? 0 : 1 },
      { char: 'i', class: 'i', count: isMobile() ? 0 : 1 },
      { char: 't', class: 't', count: isMobile() ? 0 : 1 },
      { char: 'e', class: 'e', count: isMobile() ? 0 : 1 }
    ];
    
    const positions: Position[] = [
      { top: '10%', left: '15%' },
      { top: '20%', left: '85%' },
      { top: '30%', left: '10%' },
      { top: '40%', left: '90%' },
      { top: '50%', left: '5%' },
      { top: '60%', left: '95%' },
      { top: '70%', left: '20%' },
      { top: '80%', left: '80%' },
      { top: '15%', left: '50%' },
      { top: '85%', left: '50%' }
    ];
    
    let posIndex = 0;
    
    letters.forEach(letterInfo => {
      for (let i = 0; i < letterInfo.count; i++) {
        const span = document.createElement('span');
        span.className = `${styles.floatingLetter} ${styles[letterInfo.class]}`;
        span.textContent = letterInfo.char;

        const pos = positions[posIndex % positions.length];
        span.style.top = pos.top;
        span.style.left = pos.left;
        span.style.animationDelay = Math.random() * 5 + 's';

        // Force transparent background with inline styles
        span.style.background = 'none';
        span.style.backgroundColor = 'transparent';
        span.style.border = 'none';
        span.style.boxShadow = 'none';
        span.style.outline = 'none';
        
        container.appendChild(span);
        posIndex++;
      }
    });
  };

  // Function to create spans around each character in the ASCII logo for individual dimming
  const wrapLogoCharacters = () => {
    if (!asciiLogoRef.current) return;

    const logo = asciiLogoRef.current;
    const text = logo.textContent || '';

    // Clear existing content
    logo.innerHTML = '';

    const lines = text.split('\n');
    lines.forEach((line, lineIndex) => {
      for (let charIndex = 0; charIndex < line.length; charIndex++) {
        const char = line[charIndex];
        const span = document.createElement('span');
        span.textContent = char;
        span.style.position = 'relative';
        span.style.display = 'inline-block';
        span.dataset.lineIndex = lineIndex.toString();
        span.dataset.charIndex = charIndex.toString();
        logo.appendChild(span);
      }

      // Add newline except for the last line
      if (lineIndex < lines.length - 1) {
        logo.appendChild(document.createTextNode('\n'));
      }
    });
  };

  // Function to find and dim the closest character
  const dimNearbyCharacters = (particleX: number, particleY: number) => {
    if (!asciiLogoRef.current) return;

    const spans = asciiLogoRef.current.querySelectorAll('span');
    let closestSpan: HTMLElement | null = null;
    let minDistance = Infinity;

    spans.forEach(span => {
      const lineIndex = parseInt(span.dataset.lineIndex || '0');
      const charIndex = parseInt(span.dataset.charIndex || '0');

      // Calculate character position
      const charX = charIndex * 9.6;
      const charY = lineIndex * 19.2;

      // Calculate distance from particle
      const distance = Math.sqrt(
        Math.pow(particleX - charX, 2) + Math.pow(particleY - charY, 2)
      );

      // Track the closest character
      if (distance < minDistance) {
        minDistance = distance;
        closestSpan = span as HTMLElement;
      }
    });

    // Dim only the closest character
    if (closestSpan) {
      // Remove any existing dimming class and add new one
      closestSpan.classList.remove(styles.logoDimmed);
      // Force reflow to restart animation
      closestSpan.offsetHeight;
      closestSpan.classList.add(styles.logoDimmed);

      // Remove the class after animation completes
      setTimeout(() => {
        closestSpan?.classList.remove(styles.logoDimmed);
      }, 3000);
    }
  };

  const createParticleSystem = () => {
    if (isMobile() || !logoContainerRef.current || !asciiLogoRef.current) return;

    const container = logoContainerRef.current;
    const logo = asciiLogoRef.current;
    const text = logo.textContent || '';

    // Wrap logo characters first
    wrapLogoCharacters();

    const letterPositions: LetterPosition[] = [];
    const lines = text.split('\n');

    lines.forEach((line, lineIndex) => {
      for (let charIndex = 0; charIndex < line.length; charIndex++) {
        const char = line[charIndex].toLowerCase();
        if (['d', 'e', 'x', 's', 'u', 'i', 't'].includes(char)) {
          letterPositions.push({
            char: char,
            x: charIndex * 9.6,
            y: lineIndex * 19.2,
            originalChar: char
          });
        }
      }
    });

    const colors: Record<string, string> = {
      'd': '#ffea00',
      'e': '#8be9fd',
      'x': '#50fa7b',
      's': '#bd93f9',
      'u': '#ff79c6',
      'i': '#8be9fd',
      't': '#ffea00'
    };

    const interval = setInterval(() => {
      if (letterPositions.length === 0) return;

      const numParticles = isMobile() ? 1 : Math.floor(Math.random() * 3) + 2;

      for (let i = 0; i < numParticles; i++) {
        const letterData = letterPositions[Math.floor(Math.random() * letterPositions.length)];

        // Dim nearby characters when particle is created
        dimNearbyCharacters(letterData.x, letterData.y);

        const particle = document.createElement('span');
        particle.className = styles.logoParticle;
        particle.textContent = letterData.char;
        particle.style.color = colors[letterData.char] || '#8be9fd';
        particle.style.textShadow = `0 0 20px ${colors[letterData.char] || '#8be9fd'}`;

        particle.style.left = letterData.x + 'px';
        particle.style.top = letterData.y + 'px';

        // Force transparent background with inline styles
        particle.style.background = 'none';
        particle.style.backgroundColor = 'transparent';
        particle.style.border = 'none';
        particle.style.boxShadow = 'none';
        particle.style.outline = 'none';

        const angle = Math.random() * Math.PI * 2;
        const distance = isMobile() ? 100 + Math.random() * 150 : 200 + Math.random() * 350;
        const floatX = Math.cos(angle) * distance;
        const floatY = Math.sin(angle) * distance;
        const rotation = (Math.random() - 0.5) * 720;

        particle.style.setProperty('--float-x', floatX.toString());
        particle.style.setProperty('--float-y', floatY.toString());
        particle.style.setProperty('--rotation', rotation + 'deg');

        container.appendChild(particle);

        setTimeout(() => {
          particle.remove();
        }, 6000);
      }
    }, isMobile() ? 1200 : 600);

    return () => clearInterval(interval);
  };

  const typeCode = () => {
    const lines = document.querySelectorAll(`.${styles.line}`);
    lines.forEach((line, index) => {
      const element = line as HTMLElement;
      element.style.opacity = '0';
      element.style.transform = 'translateX(-10px)';
      
      setTimeout(() => {
        element.style.transition = 'all 0.3s ease';
        element.style.opacity = '1';
        element.style.transform = 'translateX(0)';
      }, index * (isMobile() ? 30 : 50));
    });
  };

  useEffect(() => {
    const timer = setTimeout(() => {
      createBackgroundFloatingLetters();
      const cleanupParticles = createParticleSystem();
      setTimeout(typeCode, 500);

      return cleanupParticles;
    }, 100);

    return () => clearTimeout(timer);
  }, []);

  return (
    <>
      <div className={styles.floatingLetters} ref={floatingLettersRef}></div>
      
      <div className={styles.container}>
        <div className={styles.branding}>
          <div className={styles.logoContainer} ref={logoContainerRef}>
            <pre className={`${styles.logo} ${styles.logoDesktop}`} ref={asciiLogoRef}>
{`      ddd         ddd
      ddd         ddd
ddddddddd   ddddddddd   eeeeeeeee   xxx   xxx
ddd   ddd   ddd   ddd   eee          xxx xxx
ddd   ddd   ddd   ddd   eeeeeeeee     xxxxx
ddd   ddd   ddd   ddd   eee          xxx xxx
ddddddddd   ddddddddd   eeeeeeeee   xxx   xxx

sssssssss  uuu   uuu  iii  ttttttttt  eeeeeeeee
sss        uuu   uuu  iii     ttt     eee
sssssssss  uuu   uuu  iii     ttt     eeeeeeeee
      sss  uuu   uuu  iii     ttt     eee
sssssssss  uuuuuuuuu  iii     ttt     eeeeeeeee`}
            </pre>
            <pre className={`${styles.logo} ${styles.logoMobile}`}>
{`DDEX
SUITE`}
            </pre>
          </div>
          
          <p className={styles.tagline}>
            High-Performance DDEX Processing
          </p>
          
          <div className={styles.featureBadges}>
            <span className={styles.featureBadge} style={{ color: '#8be9fd' }}>10MB in &lt;100ms</span>
            <span className={styles.featureBadge} style={{ color: '#8be9fd' }}> • </span>
            <span className={styles.featureBadge} style={{ color: '#50fa7b' }}>Deterministic</span>
            <span className={styles.featureBadge} style={{ color: '#50fa7b' }}> • </span>
            <span className={styles.featureBadge} style={{ color: '#ffea00' }}>Multi-Language</span>
          </div>
          
          <div className={styles.ctaButtons}>
            <Link to="/playground" className={`${styles.ctaButton} ${styles.ctaPrimary}`}>
              Playground →
            </Link>
            <Link to="/docs/intro" className={`${styles.ctaButton} ${styles.ctaSecondary}`}>
              Documentation
            </Link>
          </div>
        </div>
        
        <div className={styles.codeWindow}>
          <div className={styles.windowHeader}>
            <div className={styles.windowControls}>
              <div className={`${styles.windowControl} ${styles.close}`}></div>
              <div className={`${styles.windowControl} ${styles.minimize}`}></div>
              <div className={`${styles.windowControl} ${styles.maximize}`}></div>
            </div>
            <div className={styles.windowTitle}>NewReleaseMessage.xml • ERN 4.3</div>
          </div>
          <div className={styles.codeContent}>
            <div className={styles.line}>
              <span className={styles.lineNumber}>1</span>
              <span className={styles.comment}>&lt;?xml version="1.0" encoding="UTF-8"?&gt;</span>
            </div>
            <div className={styles.line}>
              <span className={styles.lineNumber}>2</span>
              <span className={styles.tag}>&lt;ern:NewReleaseMessage</span>
            </div>
            <div className={styles.line}>
              <span className={styles.lineNumber}>3</span>
              <span className={styles.attrName}>  xmlns:ern=</span><span className={styles.attrValue}>"http://ddex.net/xml/ern/43"</span>
            </div>
            <div className={styles.line}>
              <span className={styles.lineNumber}>4</span>
              <span className={styles.attrName}>  xmlns:avs=</span><span className={styles.attrValue}>"http://ddex.net/xml/avs/avs"</span>
            </div>
            <div className={styles.line}>
              <span className={styles.lineNumber}>5</span>
              <span className={styles.attrName}>  MessageSchemaVersionId=</span><span className={styles.attrValue}>"ern/43"</span><span className={styles.tag}>&gt;</span>
            </div>
            <div className={styles.line}>
              <span className={styles.lineNumber}>6</span>
              <span className={styles.text}>  </span><span className={styles.tag}>&lt;MessageHeader&gt;</span>
            </div>
            <div className={styles.line}>
              <span className={styles.lineNumber}>7</span>
              <span className={styles.text}>    </span><span className={styles.tag}>&lt;MessageId&gt;</span><span className={styles.text}>MSG2025091200001</span><span className={styles.tag}>&lt;/MessageId&gt;</span>
            </div>
            <div className={styles.line}>
              <span className={styles.lineNumber}>8</span>
              <span className={styles.text}>    </span><span className={styles.tag}>&lt;MessageCreatedDateTime&gt;</span><span className={styles.text}>2025-09-12T08:00:00Z</span><span className={styles.tag}>&lt;/MessageCreatedDateTime&gt;</span>
            </div>
            <div className={styles.line}>
              <span className={styles.lineNumber}>9</span>
              <span className={styles.text}>  </span><span className={styles.tag}>&lt;/MessageHeader&gt;</span>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}