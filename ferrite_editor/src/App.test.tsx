// Simple test component to verify rendering works
function SimpleApp() {
  return (
    <div style={{
      width: '100vw',
      height: '100vh',
      backgroundColor: '#1a1a1a',
      color: 'white',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      flexDirection: 'column',
      gap: '20px'
    }}>
      <h1 style={{ fontSize: '2rem' }}>Ferrite Scene Editor</h1>
      <p>If you can see this, React is working!</p>
    </div>
  );
}

export default SimpleApp;
