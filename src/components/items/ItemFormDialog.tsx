import { useTranslation } from "react-i18next";
import type { AppItem } from "@/bindings/AppItem";
import type { LaunchItem } from "@/bindings/LaunchItem";
import type { UrlItem } from "@/bindings/UrlItem";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AppItemForm } from "./AppItemForm";
import { UrlItemForm } from "./UrlItemForm";

export type ItemFormTarget =
  { mode: "create"; type: LaunchItem["type"] } | { mode: "edit"; item: LaunchItem };

interface ItemFormDialogProps {
  target: ItemFormTarget | null;
  simConnectSupported: boolean;
  onSave: (item: LaunchItem) => void;
  onClose: () => void;
}

export function ItemFormDialog({
  target,
  simConnectSupported,
  onSave,
  onClose,
}: ItemFormDialogProps) {
  const { t } = useTranslation();

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
              simConnectSupported={simConnectSupported}
              onSubmit={saveApp}
              onCancel={onClose}
            />
          ) : (
            <UrlItemForm item={target.item} onSubmit={saveUrl} onCancel={onClose} />
          ))}

        {target?.mode === "create" && (
          <Tabs defaultValue={target.type}>
            <TabsList className="w-full">
              <TabsTrigger value="app">{t("itemForm.typeApp")}</TabsTrigger>
              <TabsTrigger value="url">{t("itemForm.typeUrl")}</TabsTrigger>
            </TabsList>
            <TabsContent value="app" className="pt-4">
              <AppItemForm
                simConnectSupported={simConnectSupported}
                onSubmit={saveApp}
                onCancel={onClose}
              />
            </TabsContent>
            <TabsContent value="url" className="pt-4">
              <UrlItemForm onSubmit={saveUrl} onCancel={onClose} />
            </TabsContent>
          </Tabs>
        )}
      </DialogContent>
    </Dialog>
  );
}
