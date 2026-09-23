import { AppWindow, Globe, Layers } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";

interface EmptyItemsProps {
  isElevated: boolean | null;
  onAddApp: () => void;
  onAddUrl: () => void;
}

export function EmptyItems({ isElevated, onAddApp, onAddUrl }: EmptyItemsProps) {
  const { t } = useTranslation();

  return (
    <Empty className="border border-dashed">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <Layers />
        </EmptyMedia>
        <EmptyTitle>{t("items.emptyTitle")}</EmptyTitle>
        <EmptyDescription>{t("items.emptyDescription")}</EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <div className="flex justify-center gap-2">
          <Button onClick={onAddApp}>
            <AppWindow />
            {t("items.addApp")}
          </Button>
          <Button variant="outline" onClick={onAddUrl}>
            <Globe />
            {t("items.addUrl")}
          </Button>
        </div>
        <p className="text-xs text-muted-foreground">
          {isElevated ? t("items.dragUnavailableAsAdmin") : t("items.dragHint")}
        </p>
      </EmptyContent>
    </Empty>
  );
}
