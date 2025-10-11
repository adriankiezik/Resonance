# Debugging White Screen Issue

The editor is compiling and running successfully, but you're seeing a white screen. Let's debug this step by step.

## Step 1: Check if the Window is Open

Run this command to see if the window exists:
```bash
ps aux | grep ferrite-editor
```

If you see the process, the app is running.

## Step 2: Open Browser DevTools

**IMPORTANT**: Tauri apps have browser dev tools!

Right-click anywhere in the white window and select "Inspect Element" or press `F12`.

Look for:
- **Console tab**: Any JavaScript errors?
- **Network tab**: Are files loading?
- **Elements tab**: Is there HTML?

## Step 3: Check What You Should See

In the Elements tab, you should see:
```html
<div id="root">
  <div class="h-screen w-screen overflow-hidden bg-background text-foreground">
    <div class="flex h-screen w-screen flex-col">
      <!-- MenuBar, Panels, etc -->
    </div>
  </div>
</div>
```

## Step 4: Common Fixes

### Fix 1: Clear Cache and Reload
In the dev tools console, run:
```javascript
location.reload(true)
```

### Fix 2: Check if JavaScript is Executing
In the dev tools console, type:
```javascript
document.getElementById('root')
```

Should return the div element, not `null`.

### Fix 3: Check if React Loaded
In the dev tools console, type:
```javascript
window.React
```

Should return an object or undefined (both are okay).

### Fix 4: Manually Test Rendering
In the dev tools console, paste:
```javascript
document.getElementById('root').innerHTML = '<h1 style="color: white; font-size: 48px;">TEST</h1>';
```

If you see "TEST", then HTML rendering works but React isn't mounting.

## Step 5: Try the Test Page

I've created a test page. Let's try it:

1. Stop the current editor (Ctrl+C)
2. Edit `src/main.tsx` and replace with:
```typescript
const root = document.getElementById('root');
if (root) {
  root.innerHTML = `
    <div style="
      width: 100vw;
      height: 100vh;
      background: #1a1a1a;
      color: white;
      display: flex;
      align-items: center;
      justify-center: center;
      flex-direction: column;
      gap: 20px;
      font-family: sans-serif;
    ">
      <h1 style="font-size: 3rem;">✅ Ferrite Editor</h1>
      <p style="font-size: 1.5rem;">React is working!</p>
      <button onclick="alert('Buttons work!')" style="
        padding: 12px 24px;
        font-size: 18px;
        border: none;
        background: #3b82f6;
        color: white;
        border-radius: 8px;
        cursor: pointer;
      ">Click Me</button>
    </div>
  `;
}
```

3. Run again: `GDK_BACKEND=x11 npm run tauri:dev`

If you see the test page, React and Tauri are working fine, and the issue is with the components.

## Step 6: Report Back

Tell me what you see:
1. What's in the Console tab?
2. What's in the Elements tab?
3. Did the test page work?
4. Any error messages?

## Quick Test Command

Run this to see everything at once:
```bash
cd /home/adrian/Projects/ferrite/ferrite_editor
GDK_BACKEND=x11 npm run tauri:dev 2>&1 | tee /tmp/editor-debug.log
```

Then press `F12` in the window and check the console.
