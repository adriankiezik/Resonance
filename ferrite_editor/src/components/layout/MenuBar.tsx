import { Button } from '../ui/button';
import { Separator } from '../ui/separator';
import { useEditorStore } from '@/stores/editorStore';
import {
  FileText,
  FolderOpen,
  Save,
  Play,
  Pause,
  Square,
  Monitor,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

export function MenuBar() {
  const { isPlaying, setIsPlaying, createScene } = useEditorStore();

  const handleNewScene = async () => {
    try {
      await createScene('Untitled Scene');
    } catch (error) {
      console.error('Failed to create scene:', error);
    }
  };

  const handleSaveScene = async () => {
    // TODO: Open file dialog
    console.log('Save scene');
  };

  const handleLoadScene = async () => {
    // TODO: Open file dialog
    console.log('Load scene');
  };

  const togglePlayMode = () => {
    setIsPlaying(!isPlaying);
  };

  const handleOpenViewport = async () => {
    try {
      await invoke('open_viewport_window');
    } catch (error) {
      console.error('Failed to open viewport window:', error);
    }
  };

  return (
    <div className="flex h-12 items-center gap-2 border-b border-border bg-muted/40 px-4">
      {/* File Operations */}
      <div className="flex items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          onClick={handleNewScene}
          title="New Scene"
        >
          <FileText className="h-4 w-4" />
          <span>New</span>
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleLoadScene}
          title="Open Scene"
        >
          <FolderOpen className="h-4 w-4" />
          <span>Open</span>
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleSaveScene}
          title="Save Scene"
        >
          <Save className="h-4 w-4" />
          <span>Save</span>
        </Button>
      </div>

      <Separator orientation="vertical" className="h-6" />

      {/* Play Controls */}
      <div className="flex items-center gap-1">
        {!isPlaying ? (
          <Button
            variant="ghost"
            size="sm"
            onClick={togglePlayMode}
            title="Play"
          >
            <Play className="h-4 w-4" />
          </Button>
        ) : (
          <>
            <Button
              variant="ghost"
              size="sm"
              onClick={togglePlayMode}
              title="Pause"
            >
              <Pause className="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsPlaying(false)}
              title="Stop"
            >
              <Square className="h-4 w-4" />
            </Button>
          </>
        )}
      </div>

      <Separator orientation="vertical" className="h-6" />

      {/* Viewport */}
      <div className="flex items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          onClick={handleOpenViewport}
          title="Open 3D Viewport"
        >
          <Monitor className="h-4 w-4" />
          <span>Viewport</span>
        </Button>
      </div>

      <Separator orientation="vertical" className="h-6" />

      {/* Scene Info */}
      <div className="flex-1 text-sm text-muted-foreground">
        Ferrite Scene Editor
      </div>
    </div>
  );
}
