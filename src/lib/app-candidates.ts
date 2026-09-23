import type { AppCandidate } from "@/bindings/AppCandidate";
import type { DroppedCandidate } from "@/bindings/DroppedCandidate";
import type { ExeInfo } from "@/bindings/ExeInfo";
import type { LaunchItem } from "@/bindings/LaunchItem";
import type { UrlCandidate } from "@/bindings/UrlCandidate";
import { appFormSchema, EMPTY_APP_FORM, type AppFormInput } from "@/components/items/app-form";
import { EMPTY_URL_FORM, urlFormSchema, type UrlFormInput } from "@/components/items/url-form";

const MAX_NAME_LENGTH = 80;

export interface ExecutableChoice {
  path: string;
  info: ExeInfo;
}

export function candidateToAppForm(candidate: AppCandidate): AppFormInput {
  return {
    ...EMPTY_APP_FORM,
    name: fitName(candidate.name),
    exePath: candidate.exePath,
    args: candidate.args ?? "",
    workingDir: candidate.workingDir ?? "",
    processName: candidate.processName,
    iconBase64: candidate.iconBase64,
  };
}

export function urlCandidateToForm(candidate: UrlCandidate): UrlFormInput {
  return { ...EMPTY_URL_FORM, name: fitName(candidate.name), url: candidate.url };
}

export function executableChoiceToAppForm({ path, info }: ExecutableChoice): AppFormInput {
  return {
    ...EMPTY_APP_FORM,
    name: fitName(info.productName),
    exePath: path,
    processName: info.processName,
    iconBase64: info.iconBase64,
  };
}

export interface CandidateGroups {
  suggested: AppCandidate[];
  others: AppCandidate[];
}

export function groupCandidates(
  candidates: readonly AppCandidate[],
  search: string,
  withSuggestions: boolean,
): CandidateGroups {
  const query = search.trim().toLowerCase();
  const matches = candidates.filter(
    (candidate) =>
      candidate.name.toLowerCase().includes(query) ||
      candidate.exePath.toLowerCase().includes(query),
  );
  if (!withSuggestions) return { suggested: [], others: matches };
  return {
    suggested: matches.filter((candidate) => candidate.suggested),
    others: matches.filter((candidate) => !candidate.suggested),
  };
}

/** Returns undefined when the candidate would not pass the item validation (e.g. an empty name). */
export function candidateToItem(candidate: DroppedCandidate): LaunchItem | undefined {
  const id = crypto.randomUUID();
  if (candidate.type === "url") {
    const parsed = urlFormSchema.safeParse(urlCandidateToForm(candidate));
    return parsed.success ? { ...parsed.data, id, type: "url" } : undefined;
  }
  const parsed = appFormSchema.safeParse(candidateToAppForm(candidate));
  return parsed.success ? { ...parsed.data, id, type: "app" } : undefined;
}

export function isAlreadyInProfile(exePath: string, profileExePaths: readonly string[]): boolean {
  const wanted = normalizePath(exePath);
  return wanted !== "" && profileExePaths.some((path) => normalizePath(path) === wanted);
}

function normalizePath(path: string): string {
  return path.trim().replaceAll("/", "\\").toLowerCase();
}

function fitName(name: string): string {
  return name.trim().slice(0, MAX_NAME_LENGTH).trim();
}
