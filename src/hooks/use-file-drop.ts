import { useEffect, useRef, useState } from "react";
import { windowEvents } from "@/lib/tauri";

interface FileDropOptions {
  enabled: boolean;
  onDrop: (paths: string[]) => void;
}

/** Dialogs rendered elsewhere (sidebar, onboarding) also block drops, without a shared registry. */
function isDialogOpen(): boolean {
  return document.querySelector('[role="dialog"], [role="alertdialog"]') !== null;
}

export function useFileDrop({ enabled, onDrop }: FileDropOptions): boolean {
  const [isDragging, setIsDragging] = useState(false);
  const latest = useRef({ enabled, onDrop });

  useEffect(() => {
    latest.current = { enabled, onDrop };
  });

  useEffect(() => {
    const accepts = () => latest.current.enabled && !isDialogOpen();
    const subscription = windowEvents.onFileDrop((event) => {
      if (event.kind === "enter") {
        setIsDragging(accepts());
        return;
      }
      setIsDragging(false);
      if (event.kind === "drop" && accepts()) latest.current.onDrop(event.paths);
    });
    return () => {
      void subscription.then((unlisten) => {
        unlisten();
      });
    };
  }, []);

  return isDragging;
}
