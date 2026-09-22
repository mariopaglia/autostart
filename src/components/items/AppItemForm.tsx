import { useEffect, useState } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { open } from "@tauri-apps/plugin-dialog";
import { ChevronDown, FolderOpen } from "lucide-react";
import { Controller, useForm, useWatch } from "react-hook-form";
import { useTranslation } from "react-i18next";
import type { AppItem } from "@/bindings/AppItem";
import type { OnClose } from "@/bindings/OnClose";
import { TextField } from "@/components/common/TextField";
import { Button } from "@/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { DialogFooter } from "@/components/ui/dialog";
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { notifyError } from "@/lib/notify";
import { commands } from "@/lib/tauri";
import { DEFAULT_DELAY_MS } from "@/schemas/profile";
import { appFormSchema, executableName, type AppFormInput, type AppFormOutput } from "./app-form";
import { ItemIcon } from "./ItemIcon";
import { ProcessNameField } from "./ProcessNameField";
import { SwitchField } from "./SwitchField";

const ON_CLOSE_OPTIONS: OnClose[] = ["graceful", "force", "keep"];

const EMPTY_APP: AppFormInput = {
  name: "",
  exePath: "",
  args: "",
  workingDir: "",
  processName: "",
  processNameMode: "auto",
  delayMs: DEFAULT_DELAY_MS,
  runAsAdmin: false,
  startMinimized: false,
  waitForSimConnect: false,
  onClose: "graceful",
  enabled: true,
};

interface AppItemFormProps {
  item?: AppItem;
  simConnectSupported: boolean;
  onSubmit: (item: AppItem) => void;
  onCancel: () => void;
}

export function AppItemForm({ item, simConnectSupported, onSubmit, onCancel }: AppItemFormProps) {
  const { t } = useTranslation();
  const [inspecting, setInspecting] = useState(false);
  const form = useForm<AppFormInput, unknown, AppFormOutput>({
    resolver: zodResolver(appFormSchema),
    defaultValues: item ?? EMPTY_APP,
  });
  const iconBase64 = useWatch({ control: form.control, name: "iconBase64" });
  const exePath = useWatch({ control: form.control, name: "exePath" });
  const [advancedOpen, setAdvancedOpen] = useState(false);

  // In automatic mode the process name follows the executable until AutoStart learns a better one.
  useEffect(() => {
    if (form.getValues("processNameMode") !== "auto") return;
    const fromExecutable = executableName(exePath);
    if (fromExecutable && form.getFieldState("exePath").isDirty) {
      form.setValue("processName", fromExecutable, { shouldValidate: true });
    }
  }, [exePath, form]);

  async function chooseExecutable() {
    const path = await open({ filters: [{ name: t("itemForm.exeFilter"), extensions: ["exe"] }] });
    if (!path) return;
    setInspecting(true);
    try {
      const info = await commands.inspectExe(path);
      form.setValue("exePath", path, { shouldValidate: true });
      form.setValue("processName", info.processName, { shouldValidate: true });
      form.setValue("processNameMode", "auto");
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
        void form.handleSubmit(
          (values) => {
            onSubmit({ ...values, id: item?.id ?? crypto.randomUUID() });
          },
          (errors) => {
            if (errors.processName || errors.workingDir) setAdvancedOpen(true);
          },
        )(event)
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
        <div className="flex flex-col gap-3">
          <SwitchField control={form.control} name="runAsAdmin" label={t("itemForm.runAsAdmin")} />
          <SwitchField
            control={form.control}
            name="startMinimized"
            label={t("itemForm.startMinimized")}
            description={t("itemForm.startMinimizedHint")}
          />
          <SwitchField
            control={form.control}
            name="waitForSimConnect"
            label={t("itemForm.waitForSimConnect")}
            description={
              simConnectSupported
                ? t("itemForm.waitForSimConnectHint")
                : t("itemForm.waitForSimConnectUnsupported")
            }
            disabled={!simConnectSupported}
          />
        </div>

        <Collapsible
          open={advancedOpen}
          onOpenChange={setAdvancedOpen}
          className="flex flex-col gap-4"
        >
          <CollapsibleTrigger asChild>
            <Button type="button" variant="ghost" size="sm" className="group w-fit">
              <ChevronDown className="transition-transform group-data-[state=open]:rotate-180" />
              {t("itemForm.advanced")}
            </Button>
          </CollapsibleTrigger>
          <CollapsibleContent className="flex flex-col gap-4">
            <TextField
              control={form.control}
              name="workingDir"
              label={t("itemForm.workingDir")}
              description={t("itemForm.workingDirHint")}
            />
            <ProcessNameField control={form.control} />
          </CollapsibleContent>
        </Collapsible>
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
