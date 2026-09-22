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
  onAddApp: () => void;
  onAddUrl: () => void;
}

export function EmptyItems({ onAddApp, onAddUrl }: EmptyItemsProps) {
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
      <EmptyContent className="flex-row justify-center">
        <Button onClick={onAddApp}>
          <AppWindow />
          {t("items.addApp")}
        </Button>
        <Button variant="outline" onClick={onAddUrl}>
          <Globe />
          {t("items.addUrl")}
        </Button>
      </EmptyContent>
    </Empty>
  );
}
