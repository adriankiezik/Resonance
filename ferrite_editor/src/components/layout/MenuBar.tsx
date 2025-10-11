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
} from 'lucide-react';

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

      {/* Scene Info */}
      <div className="flex-1 text-sm text-muted-foreground">
        Ferrite Scene Editor
      </div>
    </div>
  );
}
