import { useTranslation } from "react-i18next";
import type { AppCandidate } from "@/bindings/AppCandidate";
import { Badge } from "@/components/ui/badge";
import { CommandItem } from "@/components/ui/command";
import { shortenPath } from "@/lib/paths";
import { ItemIcon } from "./ItemIcon";

interface AppCandidateRowProps {
  candidate: AppCandidate;
  alreadyAdded: boolean;
  onSelect: () => void;
}

export function AppCandidateRow({ candidate, alreadyAdded, onSelect }: AppCandidateRowProps) {
  const { t } = useTranslation();

  return (
    <CommandItem value={`${candidate.exePath}|${candidate.args ?? ""}`} onSelect={onSelect}>
      <ItemIcon type="app" iconBase64={candidate.iconBase64} className="size-9 rounded-lg" />
      <div className="flex min-w-0 flex-1 flex-col">
        <span className="truncate font-medium">{candidate.name}</span>
        <span className="truncate text-xs text-muted-foreground" title={candidate.exePath}>
          {shortenPath(candidate.exePath)}
        </span>
      </div>
      {alreadyAdded && <Badge variant="secondary">{t("appPicker.alreadyAdded")}</Badge>}
    </CommandItem>
  );
}
