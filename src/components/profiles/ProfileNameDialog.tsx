import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Field, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { nameSchema } from "@/schemas/profile";

interface ProfileNameDialogProps {
  open: boolean;
  title: string;
  initialName: string;
  onSubmit: (name: string) => void;
  onOpenChange: (open: boolean) => void;
}

export function ProfileNameDialog({
  open,
  title,
  initialName,
  onSubmit,
  onOpenChange,
}: ProfileNameDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        {open && (
          <ProfileNameForm
            initialName={initialName}
            onSubmit={(name) => {
              onSubmit(name);
              onOpenChange(false);
            }}
            onCancel={() => {
              onOpenChange(false);
            }}
          />
        )}
      </DialogContent>
    </Dialog>
  );
}

interface ProfileNameFormProps {
  initialName: string;
  onSubmit: (name: string) => void;
  onCancel: () => void;
}

function ProfileNameForm({ initialName, onSubmit, onCancel }: ProfileNameFormProps) {
  const { t } = useTranslation();
  const [name, setName] = useState(initialName);
  const [invalid, setInvalid] = useState(false);

  return (
    <form
      className="flex flex-col gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        const result = nameSchema.safeParse(name);
        if (result.success) {
          onSubmit(result.data);
        } else {
          setInvalid(true);
        }
      }}
    >
      <Field data-invalid={invalid}>
        <FieldLabel htmlFor="profile-name">{t("profiles.nameLabel")}</FieldLabel>
        <Input
          id="profile-name"
          autoFocus
          value={name}
          aria-invalid={invalid}
          onChange={(event) => {
            setName(event.target.value);
          }}
        />
        {invalid && <FieldError>{t("validation.name")}</FieldError>}
      </Field>
      <DialogFooter>
        <Button type="button" variant="outline" onClick={onCancel}>
          {t("common.cancel")}
        </Button>
        <Button type="submit">{t("common.save")}</Button>
      </DialogFooter>
    </form>
  );
}
