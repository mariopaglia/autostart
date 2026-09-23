import { useEffect, useState } from "react";
import { FolderOpen } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { AppCandidate } from "@/bindings/AppCandidate";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandList,
} from "@/components/ui/command";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Spinner } from "@/components/ui/spinner";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  candidateToAppForm,
  executableChoiceToAppForm,
  groupCandidates,
  isAlreadyInProfile,
} from "@/lib/app-candidates";
import { notifyError } from "@/lib/notify";
import { commands } from "@/lib/tauri";
import type { AppFormInput } from "./app-form";
import { AppCandidateRow } from "./AppCandidateRow";
import { pickExecutable } from "./pick-executable";

type PickerTab = "installed" | "open";

const LOADERS: Record<PickerTab, () => Promise<AppCandidate[]>> = {
  installed: commands.listInstalledApps,
  open: commands.listOpenApps,
};

interface AppPickerDialogProps {
  open: boolean;
  profileExePaths: readonly string[];
  onPick: (initial: AppFormInput) => void;
  onClose: () => void;
}

/** Each opening fetches fresh lists, one tab at a time; `reset` forgets them when the picker closes. */
function useCandidates(open: boolean, tab: PickerTab) {
  const [lists, setLists] = useState<Partial<Record<PickerTab, AppCandidate[]>>>({});
  const loaded = lists[tab];

  useEffect(() => {
    if (!open || loaded) return;
    let cancelled = false;
    void LOADERS[tab]()
      .catch((error: unknown) => {
        notifyError(error);
        return [];
      })
      .then((candidates) => {
        if (!cancelled) setLists((current) => ({ ...current, [tab]: candidates }));
      });
    return () => {
      cancelled = true;
    };
  }, [open, tab, loaded]);

  return {
    candidates: loaded ?? null,
    reset: () => {
      setLists({});
    },
  };
}

export function AppPickerDialog({ open, profileExePaths, onPick, onClose }: AppPickerDialogProps) {
  const { t } = useTranslation();
  const [tab, setTab] = useState<PickerTab>("installed");
  const [search, setSearch] = useState("");
  const [browsing, setBrowsing] = useState(false);
  const { candidates, reset } = useCandidates(open, tab);
  const groups = groupCandidates(candidates ?? [], search, tab === "installed");

  function close() {
    reset();
    setTab("installed");
    setSearch("");
    onClose();
  }

  function pick(initial: AppFormInput) {
    close();
    onPick(initial);
  }

  async function browse() {
    setBrowsing(true);
    try {
      const choice = await pickExecutable(t("itemForm.exeFilter"));
      if (choice) pick(executableChoiceToAppForm(choice));
    } catch (error) {
      notifyError(error);
    } finally {
      setBrowsing(false);
    }
  }

  const renderGroup = (heading: string, members: AppCandidate[]) =>
    members.length > 0 && (
      <CommandGroup heading={heading}>
        {members.map((candidate) => (
          <AppCandidateRow
            key={`${candidate.exePath}|${candidate.args ?? ""}`}
            candidate={candidate}
            alreadyAdded={isAlreadyInProfile(candidate.exePath, profileExePaths)}
            onSelect={() => {
              pick(candidateToAppForm(candidate));
            }}
          />
        ))}
      </CommandGroup>
    );

  return (
    <Dialog
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen) close();
      }}
    >
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{t("appPicker.title")}</DialogTitle>
          <DialogDescription>{t("appPicker.description")}</DialogDescription>
        </DialogHeader>

        <Tabs
          value={tab}
          onValueChange={(value) => {
            setTab(value === "open" ? "open" : "installed");
          }}
        >
          <TabsList className="w-full">
            <TabsTrigger value="installed">{t("appPicker.installed")}</TabsTrigger>
            <TabsTrigger value="open">{t("appPicker.open")}</TabsTrigger>
          </TabsList>
        </Tabs>

        <Command shouldFilter={false} className="rounded-lg border">
          <CommandInput
            placeholder={t("appPicker.search")}
            value={search}
            onValueChange={setSearch}
          />
          <CommandList className="max-h-[45vh] min-h-48">
            {candidates === null ? (
              <div
                role="status"
                className="flex items-center justify-center gap-2 py-10 text-sm text-muted-foreground"
              >
                <Spinner />
                {t("appPicker.loading")}
              </div>
            ) : (
              <>
                <CommandEmpty>
                  {tab === "open" && search === ""
                    ? t("appPicker.emptyOpen")
                    : t("appPicker.empty")}
                </CommandEmpty>
                {renderGroup(t("appPicker.suggested"), groups.suggested)}
                {renderGroup(t("appPicker.apps"), groups.others)}
              </>
            )}
          </CommandList>
        </Command>

        <DialogFooter className="items-center sm:justify-between">
          <span className="text-sm text-muted-foreground">{t("appPicker.browseHint")}</span>
          <Button variant="outline" disabled={browsing} onClick={() => void browse()}>
            {browsing ? <Spinner /> : <FolderOpen />}
            {t("appPicker.browse")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
