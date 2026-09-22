import { open, save } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import type { Profile } from "@/bindings/Profile";
import { i18n } from "@/i18n";
import { notifyError, notifySuccess } from "@/lib/notify";
import { withFreshIds } from "@/lib/profile-factory";
import { commands } from "@/lib/tauri";
import { translateValidationMessage } from "@/lib/validation-messages";
import { profileSchema } from "@/schemas/profile";

const JSON_FILTER = () => [{ name: i18n.t("profiles.fileFilter"), extensions: ["json"] }];

export async function exportProfileToFile(profile: Profile) {
  const path = await save({ defaultPath: `${profile.name}.json`, filters: JSON_FILTER() });
  if (!path) return;
  try {
    await commands.exportProfile(profile.id, path);
    notifySuccess(i18n.t("profiles.exported"));
  } catch (error) {
    notifyError(error);
  }
}

/** Returns the imported profile with fresh ids, or null when cancelled or invalid. */
export async function readProfileFromFile(): Promise<Profile | null> {
  const path = await open({ filters: JSON_FILTER() });
  if (!path) return null;

  try {
    const result = profileSchema.safeParse(await commands.importProfile(path));
    if (result.success) return withFreshIds(result.data);

    const [issue] = result.error.issues;
    const field = issue?.path.join(".") ?? "";
    const message = translateValidationMessage(i18n.t, issue?.message);
    toast.error(i18n.t("profiles.importInvalid"), {
      description: i18n.t("profiles.importInvalidField", { field, message }),
    });
    return null;
  } catch (error) {
    notifyError(error);
    return null;
  }
}
