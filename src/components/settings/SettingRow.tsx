import type { ReactNode } from "react";

interface SettingRowProps {
  id: string;
  label: string;
  description?: string;
  children: ReactNode;
}

export function SettingRow({ id, label, description, children }: SettingRowProps) {
  return (
    <div className="flex items-center justify-between gap-6 py-3">
      <div className="flex min-w-0 flex-col gap-0.5">
        <label htmlFor={id} className="text-sm font-medium">
          {label}
        </label>
        {description && <p className="text-xs text-muted-foreground">{description}</p>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}
