import { useEffect, useState } from "react";
import { commands } from "@/lib/tauri";

/** `null` until the backend answers. */
export function useIsElevated(): boolean | null {
  const [isElevated, setIsElevated] = useState<boolean | null>(null);

  useEffect(() => {
    void commands.isElevated().then(setIsElevated);
  }, []);

  return isElevated;
}
