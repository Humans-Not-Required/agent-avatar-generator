import React, { useState } from 'react';

const STYLES = ['geometric', 'rings', 'robot', 'blockies', 'gradient', 'initials', 'starburst', 'mosaic', 'pixel', 'sunset'];
const API_BASE = window.location.origin;
const DEFAULT_GALLERY_SEEDS = 'nanook\nforge\ndrift\nlux\ngerundium\nsmoltbot\nclawrecipes\nagent-42';

function App() {
  const [mode, setMode] = useState('single');
  const [seed, setSeed] = useState('nanook');
  const [galleryText, setGalleryText] = useState(DEFAULT_GALLERY_SEEDS);
  const [style, setStyle] = useState('geometric');
  const [size, setSize] = useState(256);
  const [format, setFormat] = useState('png');
  const [copied, setCopied] = useState(false);
  const [galleryStyle, setGalleryStyle] = useState('all');

  const gallerySeeds = galleryText
    .split('\n')
    .map(s => s.trim())
    .filter(s => s.length > 0)
    .slice(0, 50);

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

        {/* Mode Toggle */}
        <div style={modeToggleContainerStyle}>
          <button
            onClick={() => setMode('single')}
            style={{
              ...modeButtonStyle,
              ...(mode === 'single' ? modeButtonActiveStyle : {}),
            }}
          >
            Single
          </button>
          <button
            onClick={() => setMode('gallery')}
            style={{
              ...modeButtonStyle,
              ...(mode === 'gallery' ? modeButtonActiveStyle : {}),
            }}
          >
            Gallery
          </button>
        </div>

        <div style={formStyle}>
          {mode === 'single' ? (
            /* ── Single Mode ── */
            <>
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
            </>
          ) : (
            /* ── Gallery Mode ── */
            <>
              <div style={fieldStyle}>
                <label style={labelStyle}>
                  Seeds <span style={{ color: '#666' }}>(one per line, max 50)</span>
                </label>
                <textarea
                  value={galleryText}
                  onChange={e => setGalleryText(e.target.value)}
                  placeholder="Enter seeds, one per line..."
                  rows={5}
                  style={textareaStyle}
                />
              </div>

              <div style={fieldStyle}>
                <label style={labelStyle}>Style</label>
                <div style={styleGridStyle}>
                  <button
                    onClick={() => setGalleryStyle('all')}
                    style={{
                      ...styleButtonStyle,
                      ...(galleryStyle === 'all' ? styleButtonActiveStyle : {}),
                      padding: '0.5rem 0.75rem',
                    }}
                  >
                    <span style={{ fontSize: '1.2rem' }}>✦</span>
                    <span style={{ fontSize: '0.75rem' }}>all</span>
                  </button>
                  {STYLES.map(s => (
                    <button
                      key={s}
                      onClick={() => setGalleryStyle(s)}
                      style={{
                        ...styleButtonStyle,
                        ...(galleryStyle === s ? styleButtonActiveStyle : {}),
                      }}
                    >
                      <img
                        src={`${API_BASE}/api/v1/avatar/preview?style=${s}&size=48`}
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

              {gallerySeeds.length > 0 && (
                <div style={galleryContainerStyle}>
                  <div style={{ ...gallerySeedCountStyle }}>
                    {gallerySeeds.length} avatar{gallerySeeds.length !== 1 ? 's' : ''}
                    {galleryStyle === 'all' ? ` × ${STYLES.length} styles` : ''}
                  </div>

                  {galleryStyle === 'all' ? (
                    /* All Styles: matrix view — seeds as rows, styles as columns */
                    <div style={galleryMatrixStyle}>
                      {/* Header row */}
                      <div style={galleryMatrixHeaderStyle}>
                        <div style={{ ...galleryMatrixCellStyle, fontWeight: 'bold', color: '#888' }}>seed</div>
                        {STYLES.map(s => (
                          <div key={s} style={{ ...galleryMatrixCellStyle, fontSize: '0.7rem', color: '#888' }}>
                            {s}
                          </div>
                        ))}
                      </div>
                      {/* Data rows */}
                      {gallerySeeds.map(gseed => (
                        <div key={gseed} style={galleryMatrixRowStyle}>
                          <div style={{ ...galleryMatrixCellStyle, fontSize: '0.75rem', color: '#ccc', wordBreak: 'break-all' }}>
                            {gseed}
                          </div>
                          {STYLES.map(s => (
                            <div key={`${gseed}-${s}`} style={galleryMatrixCellStyle}>
                              <img
                                src={`${API_BASE}/api/v1/avatar/${encodeURIComponent(gseed)}?style=${s}&size=${Math.min(size, 128)}`}
                                alt={`${gseed} ${s}`}
                                width={Math.min(size, 64)}
                                height={Math.min(size, 64)}
                                style={{ borderRadius: 6, display: 'block' }}
                              />
                            </div>
                          ))}
                        </div>
                      ))}
                    </div>
                  ) : (
                    /* Single Style: simple grid */
                    <div style={galleryGridStyle}>
                      {gallerySeeds.map(gseed => (
                        <div key={gseed} style={galleryItemStyle}>
                          <img
                            src={`${API_BASE}/api/v1/avatar/${encodeURIComponent(gseed)}?style=${galleryStyle}&size=${Math.min(size, 256)}`}
                            alt={`Avatar for ${gseed}`}
                            width={Math.min(size, 128)}
                            height={Math.min(size, 128)}
                            style={{ borderRadius: 8, display: 'block' }}
                          />
                          <span style={gallerySeedLabelStyle}>{gseed}</span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </>
          )}
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
  maxWidth: 800,
  width: '100%',
  boxShadow: '0 4px 20px rgba(0,0,0,0.3)',
};

const titleStyle = { margin: '0 0 0.25rem', fontSize: '1.5rem' };
const subtitleStyle = { margin: '0 0 1.5rem', color: '#888', fontSize: '0.9rem' };

const modeToggleContainerStyle = {
  display: 'flex',
  gap: '0.25rem',
  marginBottom: '1.25rem',
  background: '#0f0f1a',
  borderRadius: 8,
  padding: '0.25rem',
  width: 'fit-content',
};

const modeButtonStyle = {
  background: 'transparent',
  border: 'none',
  borderRadius: 6,
  padding: '0.4rem 1.25rem',
  color: '#888',
  cursor: 'pointer',
  fontSize: '0.9rem',
  fontWeight: 500,
  transition: 'all 0.2s',
};

const modeButtonActiveStyle = {
  background: '#0f3460',
  color: '#4a9eff',
};

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

const textareaStyle = {
  background: '#0f0f1a',
  border: '1px solid #333',
  borderRadius: 8,
  padding: '0.5rem 0.75rem',
  color: '#e0e0e0',
  fontSize: '0.9rem',
  fontFamily: 'monospace',
  outline: 'none',
  resize: 'vertical',
  lineHeight: 1.5,
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

// Gallery styles
const galleryContainerStyle = {
  marginTop: '0.5rem',
};

const gallerySeedCountStyle = {
  fontSize: '0.8rem',
  color: '#666',
  marginBottom: '0.75rem',
};

const galleryGridStyle = {
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(140px, 1fr))',
  gap: '1rem',
};

const galleryItemStyle = {
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  gap: '0.4rem',
  padding: '0.75rem',
  background: '#0f0f1a',
  borderRadius: 10,
  border: '1px solid #222',
};

const gallerySeedLabelStyle = {
  fontSize: '0.75rem',
  color: '#aaa',
  textAlign: 'center',
  wordBreak: 'break-all',
  maxWidth: '100%',
};

const galleryMatrixStyle = {
  overflowX: 'auto',
};

const galleryMatrixHeaderStyle = {
  display: 'grid',
  gridTemplateColumns: `100px repeat(${STYLES.length}, 1fr)`,
  gap: '0.4rem',
  marginBottom: '0.5rem',
  position: 'sticky',
  top: 0,
  background: '#1a1a2e',
  paddingBottom: '0.4rem',
  borderBottom: '1px solid #333',
};

const galleryMatrixRowStyle = {
  display: 'grid',
  gridTemplateColumns: `100px repeat(${STYLES.length}, 1fr)`,
  gap: '0.4rem',
  marginBottom: '0.5rem',
  alignItems: 'center',
};

const galleryMatrixCellStyle = {
  display: 'flex',
  justifyContent: 'center',
  alignItems: 'center',
  minWidth: 0,
};

const footerStyle = {
  textAlign: 'center',
  marginTop: '1.5rem',
  fontSize: '0.8rem',
  color: '#666',
};

const linkStyle = { color: '#4a9eff', textDecoration: 'none' };

export default App;
