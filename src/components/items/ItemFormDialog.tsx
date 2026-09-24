import { useTranslation } from "react-i18next";
import type { AppItem } from "@/bindings/AppItem";
import type { LaunchItem } from "@/bindings/LaunchItem";
import type { Trigger } from "@/bindings/Trigger";
import type { UrlItem } from "@/bindings/UrlItem";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { AppFormInput } from "./app-form";
import { AppItemForm } from "./AppItemForm";
import type { UrlFormInput } from "./url-form";
import { UrlItemForm } from "./UrlItemForm";

export type ItemFormTarget =
  | { mode: "create"; type: "app"; initial?: AppFormInput }
  | { mode: "create"; type: "url"; initial?: UrlFormInput }
  | { mode: "edit"; item: LaunchItem };

interface ItemFormDialogProps {
  target: ItemFormTarget | null;
  profileItems: readonly LaunchItem[];
  triggers: readonly Trigger[];
  onSave: (item: LaunchItem) => void;
  onClose: () => void;
}

function otherExePaths(items: readonly LaunchItem[], editedId: string | undefined): string[] {
  return items.flatMap((item) =>
    item.type === "app" && item.id !== editedId ? [item.exePath] : [],
  );
}

export function ItemFormDialog({
  target,
  profileItems,
  triggers,
  onSave,
  onClose,
}: ItemFormDialogProps) {
  const { t } = useTranslation();
  const exePaths = otherExePaths(
    profileItems,
    target?.mode === "edit" ? target.item.id : undefined,
  );

  function saveApp(item: AppItem) {
    onSave({ ...item, type: "app" });
  }

  function saveUrl(item: UrlItem) {
    onSave({ ...item, type: "url" });
  }

  return (
    <Dialog
      open={target !== null}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>
            {target?.mode === "edit" ? t("itemForm.editTitle") : t("itemForm.addTitle")}
          </DialogTitle>
        </DialogHeader>

        {target?.mode === "edit" &&
          (target.item.type === "app" ? (
            <AppItemForm
              item={target.item}
              otherExePaths={exePaths}
              triggers={triggers}
              onSubmit={saveApp}
              onCancel={onClose}
            />
          ) : (
            <UrlItemForm
              triggers={triggers}
              item={target.item}
              onSubmit={saveUrl}
              onCancel={onClose}
            />
          ))}

        {target?.mode === "create" && target.type === "app" && target.initial && (
          <AppItemForm
            initial={target.initial}
            otherExePaths={exePaths}
            triggers={triggers}
            onSubmit={saveApp}
            onCancel={onClose}
          />
        )}

        {target?.mode === "create" && target.type === "url" && target.initial && (
          <UrlItemForm
            triggers={triggers}
            initial={target.initial}
            onSubmit={saveUrl}
            onCancel={onClose}
          />
        )}

        {target?.mode === "create" && !target.initial && (
          <Tabs defaultValue={target.type}>
            <TabsList className="w-full">
              <TabsTrigger value="app">{t("itemForm.typeApp")}</TabsTrigger>
              <TabsTrigger value="url">{t("itemForm.typeUrl")}</TabsTrigger>
            </TabsList>
            <TabsContent value="app" className="pt-4">
              <AppItemForm
                otherExePaths={exePaths}
                triggers={triggers}
                onSubmit={saveApp}
                onCancel={onClose}
              />
            </TabsContent>
            <TabsContent value="url" className="pt-4">
              <UrlItemForm triggers={triggers} onSubmit={saveUrl} onCancel={onClose} />
            </TabsContent>
          </Tabs>
        )}
      </DialogContent>
    </Dialog>
  );
}
