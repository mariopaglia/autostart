import { toast } from "sonner";
import { i18n } from "@/i18n";
import { errorKindOf, isAppError } from "@/lib/tauri";

export function notifyError(error: unknown, title = i18n.t(`errors.${errorKindOf(error)}`)) {
  const description = isAppError(error) ? error.message : String(error);
  toast.error(title, { description });
}

export function notifySuccess(message: string) {
  toast.success(message);
}

/** `description` may hold one entry per line. */
export function notifyWarning(title: string, description: string) {
  toast.warning(title, { description, classNames: { description: "whitespace-pre-line" } });
}
