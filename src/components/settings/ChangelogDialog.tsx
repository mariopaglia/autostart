import { useTranslation } from "react-i18next";
import englishChangelog from "../../../CHANGELOG.md?raw";
import portugueseChangelog from "../../../CHANGELOG.pt-BR.md?raw";
import { MarkdownContent } from "@/components/common/MarkdownContent";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { releasedHistory } from "@/lib/changelog";

interface ChangelogDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function ChangelogDialog({ open, onOpenChange }: ChangelogDialogProps) {
  const { t, i18n } = useTranslation();
  const changelog = i18n.language === "pt-BR" ? portugueseChangelog : englishChangelog;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{t("changelog.title")}</DialogTitle>
          <DialogDescription>{t("changelog.description")}</DialogDescription>
        </DialogHeader>
        <div className="max-h-[60vh] overflow-y-auto pr-2 text-sm">
          <MarkdownContent>{releasedHistory(changelog)}</MarkdownContent>
        </div>
      </DialogContent>
    </Dialog>
  );
}
