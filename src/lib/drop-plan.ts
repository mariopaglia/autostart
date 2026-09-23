import type { DropResolution } from "@/bindings/DropResolution";
import type { LaunchItem } from "@/bindings/LaunchItem";
import type { RejectedDrop } from "@/bindings/RejectedDrop";
import type { ItemFormTarget } from "@/components/items/ItemFormDialog";
import { candidateToAppForm, candidateToItem, urlCandidateToForm } from "./app-candidates";

export type DropAction =
  | { kind: "openForm"; target: ItemFormTarget }
  | { kind: "append"; items: LaunchItem[] }
  | { kind: "none" };

export interface DropPlan {
  action: DropAction;
  rejected: RejectedDrop[];
}

/** One candidate goes through the form for review; several are added straight away. */
export function planDrop({ candidates, rejected }: DropResolution): DropPlan {
  const [first, ...others] = candidates;
  if (!first) return { action: { kind: "none" }, rejected };

  if (others.length === 0) {
    const target: ItemFormTarget =
      first.type === "app"
        ? { mode: "create", type: "app", initial: candidateToAppForm(first) }
        : { mode: "create", type: "url", initial: urlCandidateToForm(first) };
    return { action: { kind: "openForm", target }, rejected };
  }

  const items: LaunchItem[] = [];
  const invalid: RejectedDrop[] = [];
  for (const candidate of candidates) {
    const item = candidateToItem(candidate);
    if (item) {
      items.push(item);
    } else {
      const path = candidate.type === "app" ? candidate.exePath : candidate.url;
      invalid.push({ path, reason: "unsupportedFile" });
    }
  }
  const action: DropAction = items.length > 0 ? { kind: "append", items } : { kind: "none" };
  return { action, rejected: [...rejected, ...invalid] };
}
