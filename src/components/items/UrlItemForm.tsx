import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import type { z } from "zod";
import type { UrlItem } from "@/bindings/UrlItem";
import { TextField } from "@/components/common/TextField";
import { Button } from "@/components/ui/button";
import { DialogFooter } from "@/components/ui/dialog";
import { FieldGroup } from "@/components/ui/field";
import { DEFAULT_DELAY_MS, urlItemSchema } from "@/schemas/profile";

const urlFormSchema = urlItemSchema.omit({ id: true });
type UrlFormInput = z.input<typeof urlFormSchema>;
type UrlFormOutput = z.output<typeof urlFormSchema>;

const EMPTY_URL: UrlFormInput = {
  name: "",
  url: "https://",
  delayMs: DEFAULT_DELAY_MS,
  enabled: true,
};

interface UrlItemFormProps {
  item?: UrlItem;
  onSubmit: (item: UrlItem) => void;
  onCancel: () => void;
}

export function UrlItemForm({ item, onSubmit, onCancel }: UrlItemFormProps) {
  const { t } = useTranslation();
  const form = useForm<UrlFormInput, unknown, UrlFormOutput>({
    resolver: zodResolver(urlFormSchema),
    defaultValues: item ?? EMPTY_URL,
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
