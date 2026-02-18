import React, { useState, useEffect } from 'react';

const STYLES = ['geometric', 'rings', 'robot', 'blockies', 'gradient', 'initials', 'starburst'];
const API_BASE = window.location.origin;

function App() {
  const [seed, setSeed] = useState('nanook');
  const [style, setStyle] = useState('geometric');
  const [size, setSize] = useState(256);
  const [format, setFormat] = useState('png');
  const [copied, setCopied] = useState(false);

  const avatarUrl = `${API_BASE}/api/v1/avatar/${encodeURIComponent(seed)}?style=${style}&size=${size}&format=png`;
  const downloadUrl = `${API_BASE}/api/v1/avatar/${encodeURIComponent(seed)}?style=${style}&size=${size}&format=${format}`;
  const shareUrl = `${API_BASE}/avatar/view/${encodeURIComponent(seed)}?style=${style}&size=${size}`;

  const copyShareUrl = () => {
    navigator.clipboard.writeText(shareUrl).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  };

  return (
    <div style={containerStyle}>
      <div style={cardStyle}>
        <h1 style={titleStyle}>🤖 Agent Avatar Generator</h1>
        <p style={subtitleStyle}>Deterministic avatars for AI agents</p>

        <div style={formStyle}>
          <div style={fieldStyle}>
            <label style={labelStyle}>Seed</label>
            <input
              type="text"
              value={seed}
              onChange={e => setSeed(e.target.value)}
              placeholder="agent ID, email, name..."
              style={inputStyle}
            />
          </div>

          <div style={fieldStyle}>
            <label style={labelStyle}>Style</label>
            <div style={styleGridStyle}>
              {STYLES.map(s => (
                <button
                  key={s}
                  onClick={() => setStyle(s)}
                  style={{
                    ...styleButtonStyle,
                    ...(style === s ? styleButtonActiveStyle : {}),
                  }}
                >
                  <img
                    src={`${API_BASE}/api/v1/avatar/${encodeURIComponent(seed || 'preview')}?style=${s}&size=48`}
                    alt={s}
                    width={48}
                    height={48}
                    style={{ borderRadius: 6 }}
                  />
                  <span style={{ fontSize: '0.75rem' }}>{s}</span>
                </button>
              ))}
            </div>
          </div>

          <div style={fieldStyle}>
            <label style={labelStyle}>Size: {size}px</label>
            <input
              type="range"
              min={16}
              max={512}
              value={size}
              onChange={e => setSize(Number(e.target.value))}
              style={rangeStyle}
            />
          </div>

          <div style={fieldStyle}>
            <label style={labelStyle}>Format</label>
            <div style={{ display: 'flex', gap: '0.5rem' }}>
              {['png', 'svg'].map(f => (
                <button
                  key={f}
                  onClick={() => setFormat(f)}
                  style={{
                    ...formatButtonStyle,
                    ...(format === f ? formatButtonActiveStyle : {}),
                  }}
                >
                  {f.toUpperCase()}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div style={previewStyle}>
          {seed && (
            <img
              key={`${seed}-${style}-${size}`}
              src={avatarUrl}
              alt={`Avatar for ${seed}`}
              style={{ borderRadius: 12, maxWidth: '100%' }}
              width={Math.min(size, 400)}
              height={Math.min(size, 400)}
            />
          )}
        </div>

        <div style={actionsStyle}>
          <a href={downloadUrl} download={`${seed || 'avatar'}.${format}`} style={buttonStyle}>
            ⬇ Download {format.toUpperCase()}
          </a>
          <button onClick={copyShareUrl} style={buttonStyle}>
            {copied ? '✅ Copied!' : '🔗 Copy Share URL'}
          </button>
        </div>

        <p style={footerStyle}>
          <a href="/api/v1/openapi.json" style={linkStyle}>OpenAPI</a>
          {' · '}
          <a href="/llms.txt" style={linkStyle}>llms.txt</a>
          {' · '}
          <a href="https://github.com/Humans-Not-Required/agent-avatar-generator" style={linkStyle}>GitHub</a>
        </p>
      </div>
    </div>
  );
}

// ── Styles ──

const containerStyle = {
  fontFamily: 'system-ui, -apple-system, sans-serif',
  display: 'flex',
  justifyContent: 'center',
  alignItems: 'center',
  minHeight: '100vh',
  margin: 0,
  background: '#0f0f1a',
  color: '#e0e0e0',
  padding: '1rem',
};

const cardStyle = {
  background: '#1a1a2e',
  borderRadius: 16,
  padding: '2rem',
  maxWidth: 600,
  width: '100%',
  boxShadow: '0 4px 20px rgba(0,0,0,0.3)',
};

const titleStyle = { margin: '0 0 0.25rem', fontSize: '1.5rem' };
const subtitleStyle = { margin: '0 0 1.5rem', color: '#888', fontSize: '0.9rem' };

const formStyle = { display: 'flex', flexDirection: 'column', gap: '1rem' };
const fieldStyle = { display: 'flex', flexDirection: 'column', gap: '0.25rem' };
const labelStyle = { fontSize: '0.85rem', color: '#aaa' };

const inputStyle = {
  background: '#0f0f1a',
  border: '1px solid #333',
  borderRadius: 8,
  padding: '0.5rem 0.75rem',
  color: '#e0e0e0',
  fontSize: '1rem',
  outline: 'none',
};

const styleGridStyle = {
  display: 'flex',
  gap: '0.5rem',
  flexWrap: 'wrap',
};

const styleButtonStyle = {
  background: '#0f0f1a',
  border: '2px solid #333',
  borderRadius: 8,
  padding: '0.5rem',
  cursor: 'pointer',
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  gap: '0.25rem',
  color: '#aaa',
  transition: 'border-color 0.2s',
};

const styleButtonActiveStyle = {
  borderColor: '#4a9eff',
  color: '#4a9eff',
};

const rangeStyle = { width: '100%', accentColor: '#4a9eff' };

const formatButtonStyle = {
  background: '#0f0f1a',
  border: '1px solid #333',
  borderRadius: 6,
  padding: '0.4rem 1rem',
  color: '#aaa',
  cursor: 'pointer',
  fontSize: '0.85rem',
};

const formatButtonActiveStyle = {
  borderColor: '#4a9eff',
  color: '#4a9eff',
  background: '#1a2a4a',
};

const previewStyle = {
  display: 'flex',
  justifyContent: 'center',
  margin: '1.5rem 0',
  minHeight: 100,
};

const actionsStyle = {
  display: 'flex',
  gap: '0.5rem',
  justifyContent: 'center',
  flexWrap: 'wrap',
};

const buttonStyle = {
  background: '#0f3460',
  color: '#e0e0e0',
  padding: '0.5rem 1rem',
  borderRadius: 8,
  textDecoration: 'none',
  fontSize: '0.9rem',
  border: 'none',
  cursor: 'pointer',
  transition: 'background 0.2s',
};

const footerStyle = {
  textAlign: 'center',
  marginTop: '1.5rem',
  fontSize: '0.8rem',
  color: '#666',
};

const linkStyle = { color: '#4a9eff', textDecoration: 'none' };

export default App;
