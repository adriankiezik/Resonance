// Ultra-minimal React test
import { createRoot } from 'react-dom/client';

const App = () => {
  return (
    <div style={{
      width: '100vw',
      height: '100vh',
      backgroundColor: '#1a1a2e',
      color: 'white',
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      fontFamily: 'system-ui, sans-serif',
      gap: '20px'
    }}>
      <h1 style={{ fontSize: '3rem', margin: 0 }}>✅ Ferrite Editor</h1>
      <p style={{ fontSize: '1.5rem', opacity: 0.8 }}>React + Tauri Working!</p>
      <button
        onClick={() => alert('Button works!')}
        style={{
          padding: '12px 24px',
          fontSize: '16px',
          background: '#6366f1',
          color: 'white',
          border: 'none',
          borderRadius: '8px',
          cursor: 'pointer'
        }}
      >
        Click Me
      </button>
    </div>
  );
};

const root = document.getElementById('root');
if (root) {
  createRoot(root).render(<App />);
} else {
  console.error('Root element not found!');
}
