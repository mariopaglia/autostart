import {
  closestCenter,
  DndContext,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import { restrictToVerticalAxis } from "@dnd-kit/modifiers";
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import type { LaunchItem } from "@/bindings/LaunchItem";
import type { Trigger } from "@/bindings/Trigger";
import { ItemCard } from "./ItemCard";

interface ItemListProps {
  profileId: string;
  items: LaunchItem[];
  triggers: readonly Trigger[];
  missingExecutables: ReadonlySet<string>;
  onChange: (items: LaunchItem[]) => void;
  onEdit: (item: LaunchItem) => void;
}

export function ItemList({
  profileId,
  items,
  triggers,
  missingExecutables,
  onChange,
  onEdit,
}: ItemListProps) {
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  function handleDragEnd({ active, over }: DragEndEvent) {
    if (!over || active.id === over.id) return;
    const from = items.findIndex((item) => item.id === active.id);
    const to = items.findIndex((item) => item.id === over.id);
    onChange(arrayMove(items, from, to));
  }

  function replace(updated: LaunchItem) {
    onChange(items.map((item) => (item.id === updated.id ? updated : item)));
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      modifiers={[restrictToVerticalAxis]}
      onDragEnd={handleDragEnd}
    >
      <SortableContext items={items.map((item) => item.id)} strategy={verticalListSortingStrategy}>
        <ul className="flex flex-col gap-2">
          {items.map((item) => (
            <ItemCard
              key={item.id}
              profileId={profileId}
              item={item}
              triggers={triggers}
              executableMissing={item.type === "app" && missingExecutables.has(item.exePath)}
              onToggle={(enabled) => {
                replace({ ...item, enabled });
              }}
              onEdit={() => {
                onEdit(item);
              }}
              onRemove={() => {
                onChange(items.filter((candidate) => candidate.id !== item.id));
              }}
            />
          ))}
        </ul>
      </SortableContext>
    </DndContext>
  );
}
