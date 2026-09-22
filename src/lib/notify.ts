import { toast } from "sonner";
import { i18n } from "@/i18n";
import { errorKindOf, isAppError } from "@/lib/tauri";

export function notifyError(error: unknown) {
  const description = isAppError(error) ? error.message : String(error);
  toast.error(i18n.t(`errors.${errorKindOf(error)}`), { description });
}

export function notifySuccess(message: string) {
  toast.success(message);
}
