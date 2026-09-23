import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import type { UrlItem } from "@/bindings/UrlItem";
import { TextField } from "@/components/common/TextField";
import { Button } from "@/components/ui/button";
import { DialogFooter } from "@/components/ui/dialog";
import { FieldGroup } from "@/components/ui/field";
import { EMPTY_URL_FORM, urlFormSchema, type UrlFormInput, type UrlFormOutput } from "./url-form";

interface UrlItemFormProps {
  item?: UrlItem;
  initial?: UrlFormInput;
  onSubmit: (item: UrlItem) => void;
  onCancel: () => void;
}

export function UrlItemForm({ item, initial, onSubmit, onCancel }: UrlItemFormProps) {
  const { t } = useTranslation();
  const form = useForm<UrlFormInput, unknown, UrlFormOutput>({
    resolver: zodResolver(urlFormSchema),
    defaultValues: item ?? initial ?? EMPTY_URL_FORM,
  });

  return (
    <form
      className="flex flex-col gap-5"
      onSubmit={(event) =>
        void form.handleSubmit((values) => {
          onSubmit({ ...values, id: item?.id ?? crypto.randomUUID() });
        })(event)
      }
    >
      <FieldGroup>
        <TextField control={form.control} name="name" label={t("itemForm.name")} />
        <TextField
          control={form.control}
          name="url"
          label={t("itemForm.url")}
          inputProps={{ type: "url", placeholder: "https://" }}
        />
        <TextField
          control={form.control}
          name="delayMs"
          label={t("itemForm.delay")}
          numeric
          inputProps={{ min: 0, max: 60_000, step: 100 }}
        />
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
