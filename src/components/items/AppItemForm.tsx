import { useEffect, useState } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { ChevronDown, CopyCheck, FolderOpen } from "lucide-react";
import { Controller, useForm, useWatch } from "react-hook-form";
import { useTranslation } from "react-i18next";
import type { AppItem } from "@/bindings/AppItem";
import type { OnClose } from "@/bindings/OnClose";
import type { Trigger } from "@/bindings/Trigger";
import { TextField } from "@/components/common/TextField";
import { Button } from "@/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { DialogFooter } from "@/components/ui/dialog";
import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { isAlreadyInProfile } from "@/lib/app-candidates";
import { notifyError } from "@/lib/notify";
import { supportsSimConnect } from "@/lib/trigger-presets";
import {
  appFormSchema,
  EMPTY_APP_FORM,
  executableName,
  type AppFormInput,
  type AppFormOutput,
} from "./app-form";
import { ItemIcon } from "./ItemIcon";
import { OnlyForTriggersField } from "./OnlyForTriggersField";
import { pickExecutable } from "./pick-executable";
import { ProcessNameField } from "./ProcessNameField";
import { SwitchField } from "./SwitchField";

const ON_CLOSE_OPTIONS: OnClose[] = ["graceful", "force", "keep"];

interface AppItemFormProps {
  item?: AppItem;
  initial?: AppFormInput;
  otherExePaths: readonly string[];
  triggers: readonly Trigger[];
  onSubmit: (item: AppItem) => void;
  onCancel: () => void;
}

export function AppItemForm({
  item,
  initial,
  otherExePaths,
  triggers,
  onSubmit,
  onCancel,
}: AppItemFormProps) {
  const { t } = useTranslation();
  const [inspecting, setInspecting] = useState(false);
  const form = useForm<AppFormInput, unknown, AppFormOutput>({
    resolver: zodResolver(appFormSchema),
    defaultValues: item ?? initial ?? EMPTY_APP_FORM,
  });
  const iconBase64 = useWatch({ control: form.control, name: "iconBase64" });
  const exePath = useWatch({ control: form.control, name: "exePath" });
  const isDuplicate = isAlreadyInProfile(exePath, otherExePaths);
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const simConnectSupported = supportsSimConnect(triggers);

  // In automatic mode the process name follows the executable until AutoStart learns a better one.
  useEffect(() => {
    if (form.getValues("processNameMode") !== "auto") return;
    const fromExecutable = executableName(exePath);
    if (fromExecutable && form.getFieldState("exePath").isDirty) {
      form.setValue("processName", fromExecutable, { shouldValidate: true });
    }
  }, [exePath, form]);

  async function chooseExecutable() {
    setInspecting(true);
    try {
      const choice = await pickExecutable(t("itemForm.exeFilter"));
      if (!choice) return;
      const { path, info } = choice;
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
        <div className="flex flex-col gap-2">
          <TextField
            control={form.control}
            name="exePath"
            label={t("itemForm.exePath")}
            inputProps={{ placeholder: "C:\\Program Files\\App\\App.exe" }}
          />
          {isDuplicate && (
            <FieldDescription
              role="status"
              className="flex items-center gap-1.5 text-amber-700 dark:text-amber-400"
            >
              <CopyCheck className="size-4 shrink-0" />
              {t("itemForm.duplicate")}
            </FieldDescription>
          )}
        </div>
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
        {triggers.length > 1 && (
          <Controller
            control={form.control}
            name="onlyForTriggers"
            render={({ field }) => (
              <OnlyForTriggersField
                triggers={triggers}
                value={field.value ?? []}
                onChange={field.onChange}
              />
            )}
          />
        )}
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
            onToggle={(checked) => {
              if (checked) form.setValue("launchBeforeSimulator", false);
            }}
          />
          <SwitchField
            control={form.control}
            name="launchBeforeSimulator"
            label={t("itemForm.launchBeforeSimulator")}
            description={t("itemForm.launchBeforeSimulatorHint")}
            onToggle={(checked) => {
              if (checked) form.setValue("waitForSimConnect", false);
            }}
          />
          <SwitchField
            control={form.control}
            name="restartOnCrash"
            label={t("itemForm.restartOnCrash")}
            description={t("itemForm.restartOnCrashHint")}
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
