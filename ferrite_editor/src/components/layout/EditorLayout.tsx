import { MenuBar } from './MenuBar';
import { SceneHierarchy } from '../hierarchy/SceneHierarchy';
import { Inspector } from '../inspector/Inspector';
import { Viewport3D } from '../viewport/Viewport3D';
import { Panel, PanelGroup, PanelResizeHandle } from 'react-resizable-panels';

export function EditorLayout() {
  return (
    <div className="flex h-screen w-screen flex-col">
      <MenuBar />

      <PanelGroup direction="horizontal" className="flex-1">
        {/* Left Panel - Scene Hierarchy */}
        <Panel defaultSize={20} minSize={15} maxSize={30}>
          <SceneHierarchy />
        </Panel>

        <PanelResizeHandle className="w-1 bg-border hover:bg-primary/50 transition-colors" />

        {/* Center Panel - Viewport */}
        <Panel defaultSize={60} minSize={40}>
          <Viewport3D />
        </Panel>

        <PanelResizeHandle className="w-1 bg-border hover:bg-primary/50 transition-colors" />

        {/* Right Panel - Inspector */}
        <Panel defaultSize={20} minSize={15} maxSize={30}>
          <Inspector />
        </Panel>
      </PanelGroup>
    </div>
  );
}
