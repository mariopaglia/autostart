import { useState } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen } from "lucide-react";
import { Controller, useForm, useWatch } from "react-hook-form";
import { useTranslation } from "react-i18next";
import type { z } from "zod";
import type { AppItem } from "@/bindings/AppItem";
import type { OnClose } from "@/bindings/OnClose";
import { TextField } from "@/components/common/TextField";
import { Button } from "@/components/ui/button";
import { DialogFooter } from "@/components/ui/dialog";
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { notifyError } from "@/lib/notify";
import { commands } from "@/lib/tauri";
import { appItemSchema, DEFAULT_DELAY_MS } from "@/schemas/profile";
import { ItemIcon } from "./ItemIcon";

const appFormSchema = appItemSchema.omit({ id: true });
type AppFormInput = z.input<typeof appFormSchema>;
type AppFormOutput = z.output<typeof appFormSchema>;

const ON_CLOSE_OPTIONS: OnClose[] = ["graceful", "force", "keep"];

const EMPTY_APP: AppFormInput = {
  name: "",
  exePath: "",
  args: "",
  workingDir: "",
  processName: "",
  delayMs: DEFAULT_DELAY_MS,
  runAsAdmin: false,
  onClose: "graceful",
  enabled: true,
};

interface AppItemFormProps {
  item?: AppItem;
  onSubmit: (item: AppItem) => void;
  onCancel: () => void;
}

export function AppItemForm({ item, onSubmit, onCancel }: AppItemFormProps) {
  const { t } = useTranslation();
  const [inspecting, setInspecting] = useState(false);
  const form = useForm<AppFormInput, unknown, AppFormOutput>({
    resolver: zodResolver(appFormSchema),
    defaultValues: item ?? EMPTY_APP,
  });
  const iconBase64 = useWatch({ control: form.control, name: "iconBase64" });

  async function chooseExecutable() {
    const path = await open({ filters: [{ name: t("itemForm.exeFilter"), extensions: ["exe"] }] });
    if (!path) return;
    setInspecting(true);
    try {
      const info = await commands.inspectExe(path);
      form.setValue("exePath", path, { shouldValidate: true });
      form.setValue("processName", info.processName, { shouldValidate: true });
      form.setValue("iconBase64", info.iconBase64);
      if (!form.getValues("name")) {
        form.setValue("name", info.productName, { shouldValidate: true });
      }
    } catch (error) {
      notifyError(error);
    } finally {
      setInspecting(false);
    }
  }

  return (
    <form
      className="flex flex-col gap-5"
      onSubmit={(event) =>
        void form.handleSubmit((values) => {
          onSubmit({ ...values, id: item?.id ?? crypto.randomUUID() });
        })(event)
      }
    >
      <div className="flex items-center gap-3">
        <ItemIcon type="app" iconBase64={iconBase64} />
        <Button
          type="button"
          variant="outline"
          disabled={inspecting}
          onClick={() => void chooseExecutable()}
        >
          <FolderOpen />
          {t("itemForm.chooseExe")}
        </Button>
      </div>

      <FieldGroup>
        <TextField control={form.control} name="name" label={t("itemForm.name")} />
        <TextField
          control={form.control}
          name="exePath"
          label={t("itemForm.exePath")}
          inputProps={{ placeholder: "C:\\Program Files\\App\\App.exe" }}
        />
        <TextField
          control={form.control}
          name="processName"
          label={t("itemForm.processName")}
          description={t("itemForm.processNameHint")}
        />
        <div className="grid grid-cols-2 gap-4">
          <TextField
            control={form.control}
            name="args"
            label={t("itemForm.args")}
            description={t("itemForm.argsHint")}
          />
          <TextField
            control={form.control}
            name="delayMs"
            label={t("itemForm.delay")}
            numeric
            inputProps={{ min: 0, max: 60_000, step: 100 }}
          />
        </div>
        <TextField
          control={form.control}
          name="workingDir"
          label={t("itemForm.workingDir")}
          description={t("itemForm.workingDirHint")}
        />
        <div className="grid grid-cols-2 gap-4">
          <Controller
            control={form.control}
            name="onClose"
            render={({ field }) => (
              <Field>
                <FieldLabel htmlFor="onClose">{t("itemForm.onClose")}</FieldLabel>
                <Select value={field.value} onValueChange={field.onChange}>
                  <SelectTrigger id="onClose" className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {ON_CLOSE_OPTIONS.map((option) => (
                      <SelectItem key={option} value={option}>
                        {t(`itemForm.onCloseOptions.${option}`)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </Field>
            )}
          />
          <Controller
            control={form.control}
            name="runAsAdmin"
            render={({ field }) => (
              <Field orientation="horizontal" className="self-end pb-2">
                <Switch id="runAsAdmin" checked={field.value} onCheckedChange={field.onChange} />
                <FieldLabel htmlFor="runAsAdmin">{t("itemForm.runAsAdmin")}</FieldLabel>
              </Field>
            )}
          />
        </div>
      </FieldGroup>

      <DialogFooter>
        <Button type="button" variant="outline" onClick={onCancel}>
          {t("common.cancel")}
        </Button>
        <Button type="submit">{t("common.save")}</Button>
      </DialogFooter>
    </form>
  );
}
